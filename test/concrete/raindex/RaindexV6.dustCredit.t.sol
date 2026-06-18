// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";
import {Float, LibDecimalFloat} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";
import {LibRainDeploy} from "rain-deploy-0.1.2/src/lib/LibRainDeploy.sol";
import {LibTOFUTokenDecimals} from "rain-tofu-erc20-decimals-0.1.1/src/lib/LibTOFUTokenDecimals.sol";
import {RaindexV6DustCreditHarness} from "test/concrete/raindex/RaindexV6DustCreditHarness.sol";

/// @title RaindexV6DustCreditTest
/// @notice L01 follow-up (#2673): every lossy float->fixed-decimal token move
/// leaves a sub-base-unit remainder. `pullTokens` rounds the transfer UP (the
/// orderbook receives the excess) and `pushTokens` rounds it DOWN (the orderbook
/// keeps the shortfall); in both cases the residue is real tokens the orderbook
/// holds on behalf of `account`. Rather than stranding it, the residue is booked
/// into `sDustCredit[account][token]` (token units) and realized into the next
/// move for the same `(account, token)`.
///
/// The solvency invariant is paramount: the orderbook's real token balance must
/// always cover every vault obligation plus every tracked credit, and a credit
/// can only ever release tokens the orderbook provably already holds for that
/// user, so no move can over-credit or dip into a vault balance. The conservation
/// invariant exercised throughout is, in token units:
///
///   realBalanceDelta == netRequested + credit
///
/// where `netRequested` is the sum of requested pull amounts minus push amounts
/// (the exact economic value the vault accounting moved) and `credit` is the
/// standing ledger entry. It holds exactly after every move.
contract RaindexV6DustCreditTest is Test {
    using LibDecimalFloat for Float;

    RaindexV6DustCreditHarness internal harness;
    address internal owner = makeAddr("owner");
    // 6-decimal token: one base unit is 1e-6 token units.
    MockToken internal token;

    function setUp() external {
        // pullTokens/pushTokens read token decimals via the deployed TOFU
        // contract, so it must exist before any move.
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);

        // Etch the runtime code so the large RaindexV6-derived harness isn't
        // subject to the EIP-170 creation size limit.
        harness = RaindexV6DustCreditHarness(payable(address(uint160(uint256(keccak256("dust.credit.harness"))))));
        vm.etch(address(harness), type(RaindexV6DustCreditHarness).runtimeCode);

        token = new MockToken("Token", "TKN", 6);
        // Both sides are funded generously so transfers never fail for lack of
        // balance/approval; the tests assert exact deltas, not absolute balances.
        token.mint(owner, 1e18);
        token.mint(address(harness), 1e18);
        vm.prank(owner);
        token.approve(address(harness), type(uint256).max);
    }

    /// At rest the ledger is empty.
    function testDustCreditStartsZero() external view {
        assertTrue(harness.exposedDustCredit(owner, address(token)).isZero(), "credit starts zero");
    }

    /// Two consecutive pulls of 1.5 base units. The first is lossy and rounds the
    /// transfer up to 2, booking 0.5 base units of credit. The second consumes
    /// that credit: it only needs to pull 1.0 base unit (lossless), so exactly 1
    /// is transferred and the ledger returns to zero. Total moved is 3 base units
    /// for 3.0 requested - the round-up dust is recovered, not stranded.
    /// Pins the realize-into-next-pull path: a mutation that ignores the standing
    /// credit pulls 2 again on the second move (4 total) and leaves a stale
    /// credit.
    function testPullRealizesCreditIntoNextPull() external {
        // 1.5 base units = 15e-7 token units.
        Float amount = LibDecimalFloat.packLossless(15, -7);

        (uint256 moved1,) = harness.exposedPull(owner, address(token), amount);
        assertEq(moved1, 2, "first pull rounds up to 2");
        // Credit is 2 - 1.5 = 0.5 base units = 5e-7 token units.
        assertTrue(
            harness.exposedDustCredit(owner, address(token)).eq(LibDecimalFloat.packLossless(5, -7)), "credit 0.5"
        );

        (uint256 moved2,) = harness.exposedPull(owner, address(token), amount);
        assertEq(moved2, 1, "second pull consumes credit and moves only 1");
        assertTrue(harness.exposedDustCredit(owner, address(token)).isZero(), "credit cleared");

        // Exactly 3 base units moved into the harness for 3.0 requested.
        assertEq(moved1 + moved2, 3, "exact conservation across two pulls");
    }

    /// Two consecutive pushes of 1.5 base units. The first rounds the transfer
    /// down to 1, booking 0.5 base units of credit (owed to the user). The second
    /// realizes it: 1.5 + 0.5 = 2.0 base units, so exactly 2 are transferred and
    /// the ledger returns to zero. Total moved is 3 base units for 3.0 owed - the
    /// round-down dust is paid out, not stranded.
    /// Pins the realize-into-next-push path: a mutation that ignores the standing
    /// credit pushes 1 again on the second move (2 total) and leaves a stale
    /// credit.
    function testPushRealizesCreditIntoNextPush() external {
        Float amount = LibDecimalFloat.packLossless(15, -7);

        (uint256 moved1,) = harness.exposedPush(owner, address(token), amount);
        assertEq(moved1, 1, "first push rounds down to 1");
        assertTrue(
            harness.exposedDustCredit(owner, address(token)).eq(LibDecimalFloat.packLossless(5, -7)), "credit 0.5"
        );

        (uint256 moved2,) = harness.exposedPush(owner, address(token), amount);
        assertEq(moved2, 2, "second push realizes the whole-unit credit");
        assertTrue(harness.exposedDustCredit(owner, address(token)).isZero(), "credit cleared");

        assertEq(moved1 + moved2, 3, "exact conservation across two pushes");
    }

    /// Sub-base-unit pushes accumulate until they cross a whole base unit. Three
    /// pushes of 0.4 base units each: 0.4 -> 0.8 -> 1.2. Only the third move
    /// transfers a token (1 base unit), leaving 0.2 base units of credit. The
    /// orderbook never over-pays: it pays a whole unit only once the owed
    /// fraction has provably accumulated past one.
    /// Pins the sub-unit accumulation crossing a whole unit: a mutation that
    /// resets the credit each move never reaches the boundary and transfers 0
    /// three times.
    function testPushAccumulatesSubUnitToWholeUnit() external {
        // 0.4 base units = 4e-7 token units.
        Float amount = LibDecimalFloat.packLossless(4, -7);

        (uint256 moved1,) = harness.exposedPush(owner, address(token), amount);
        assertEq(moved1, 0, "0.4 pushes nothing");
        assertTrue(
            harness.exposedDustCredit(owner, address(token)).eq(LibDecimalFloat.packLossless(4, -7)), "credit 0.4"
        );

        (uint256 moved2,) = harness.exposedPush(owner, address(token), amount);
        assertEq(moved2, 0, "0.8 pushes nothing");
        assertTrue(
            harness.exposedDustCredit(owner, address(token)).eq(LibDecimalFloat.packLossless(8, -7)), "credit 0.8"
        );

        (uint256 moved3,) = harness.exposedPush(owner, address(token), amount);
        assertEq(moved3, 1, "1.2 crosses a whole unit, pushes 1");
        assertTrue(
            harness.exposedDustCredit(owner, address(token)).eq(LibDecimalFloat.packLossless(2, -7)), "credit 0.2"
        );
    }

    /// A pull whose amount is smaller than the standing credit moves no tokens at
    /// all and only shrinks the credit. First a 1.5 base-unit pull books 0.5 base
    /// units of credit; then a 0.3 base-unit pull is fully covered by that credit,
    /// so nothing is pulled and the credit drops to 0.2 base units. This is the
    /// conservative "credit covers the whole pull" branch.
    /// Pins the no-transfer credit-shrink path: a mutation that always pulls at
    /// least one base unit (ignoring the covering credit) moves a token here and
    /// over-charges the owner.
    function testPullCoveredEntirelyByCredit() external {
        // Seed 0.5 base units of credit with a 1.5 base-unit pull.
        (uint256 seeded,) = harness.exposedPull(owner, address(token), LibDecimalFloat.packLossless(15, -7));
        assertEq(seeded, 2, "seed pull moves 2");
        assertTrue(
            harness.exposedDustCredit(owner, address(token)).eq(LibDecimalFloat.packLossless(5, -7)), "seed credit 0.5"
        );

        uint256 ownerBefore = token.balanceOf(owner);
        // 0.3 base units < 0.5 credit, so no tokens move.
        (uint256 moved,) = harness.exposedPull(owner, address(token), LibDecimalFloat.packLossless(3, -7));
        assertEq(moved, 0, "covered pull moves nothing");
        assertEq(token.balanceOf(owner), ownerBefore, "owner balance unchanged");
        // Credit shrinks by the requested 0.3 to 0.2 base units.
        assertTrue(
            harness.exposedDustCredit(owner, address(token)).eq(LibDecimalFloat.packLossless(2, -7)), "credit 0.2"
        );
    }

    /// A push that realizes accumulated credit DOES pay an extra base unit out of
    /// the orderbook. Seed 0.8 base units of credit (two 0.4 pushes), then a push
    /// of 0.5 base units: 0.5 + 0.8 = 1.3, so the orderbook transfers 1 base unit
    /// it would otherwise have truncated away, and 0.3 base units of credit
    /// remain. Confirms credit can only ADD to a push (never subtract), keeping it
    /// solvent.
    function testPushCreditAddsExtraUnit() external {
        harness.exposedPush(owner, address(token), LibDecimalFloat.packLossless(4, -7));
        harness.exposedPush(owner, address(token), LibDecimalFloat.packLossless(4, -7));
        assertTrue(
            harness.exposedDustCredit(owner, address(token)).eq(LibDecimalFloat.packLossless(8, -7)), "credit 0.8"
        );

        uint256 ownerBefore = token.balanceOf(owner);
        // 0.5 + 0.8 = 1.3 base units -> transfer 1, keep 0.3.
        (uint256 moved,) = harness.exposedPush(owner, address(token), LibDecimalFloat.packLossless(5, -7));
        assertEq(moved, 1, "0.5 push with 0.8 credit transfers a realized unit");
        assertEq(token.balanceOf(owner) - ownerBefore, 1, "owner received exactly 1 base unit");
        assertTrue(
            harness.exposedDustCredit(owner, address(token)).eq(LibDecimalFloat.packLossless(3, -7)), "credit 0.3"
        );
    }

    /// A lossless move never touches the ledger. A 3-base-unit pull (3e-6, exact
    /// at 6 decimals) transfers exactly 3 and leaves the credit at zero. Pins that
    /// the ledger is only written when there is a genuine remainder.
    function testLosslessMoveLeavesCreditZero() external {
        (uint256 moved,) = harness.exposedPull(owner, address(token), LibDecimalFloat.packLossless(3, -6));
        assertEq(moved, 3, "lossless pull moves exactly 3");
        assertTrue(harness.exposedDustCredit(owner, address(token)).isZero(), "lossless leaves credit zero");
    }

    /// The credit ledger is keyed per `(user, token)`: a remainder accrued for one
    /// user is never visible to or realized by another. Two different owners each
    /// pulling 1.5 base units both round up independently to 2 and each carry their
    /// own 0.5 base-unit credit. Pins the mapping key so a mutation collapsing the
    /// ledger to a single global slot is caught.
    function testCreditIsolatedPerUser() external {
        address other = makeAddr("other");
        token.mint(other, 1e18);
        vm.prank(other);
        token.approve(address(harness), type(uint256).max);

        Float amount = LibDecimalFloat.packLossless(15, -7);
        (uint256 movedOwner,) = harness.exposedPull(owner, address(token), amount);
        (uint256 movedOther,) = harness.exposedPull(other, address(token), amount);

        assertEq(movedOwner, 2, "owner rounds up independently");
        assertEq(movedOther, 2, "other rounds up independently");
        assertTrue(
            harness.exposedDustCredit(owner, address(token)).eq(LibDecimalFloat.packLossless(5, -7)), "owner credit"
        );
        assertTrue(
            harness.exposedDustCredit(other, address(token)).eq(LibDecimalFloat.packLossless(5, -7)), "other credit"
        );
    }

    /// Fuzz: an arbitrary sequence of pulls and pushes preserves the conservation
    /// and solvency invariants after EVERY move.
    ///
    /// - Conservation (exact): the harness's real token-balance delta in token
    ///   units equals `netRequested + credit`. Every base unit that physically
    ///   moved is accounted for by the requested economic value plus the standing
    ///   credit - no dust is created or destroyed.
    /// - Solvency: the credit is always non-negative and strictly less than one
    ///   base unit, so the orderbook can never over-credit (a credit is always
    ///   backed by real tokens it already holds) and never strands a whole base
    ///   unit it should have realized.
    /// forge-config: default.fuzz.runs = 2000
    function testConservationAndSolvencyInvariant(bool[] memory isPull, uint8[] memory units) external {
        vm.assume(isPull.length > 0);
        uint256 n = isPull.length < units.length ? isPull.length : units.length;
        vm.assume(n > 0);

        // One base unit is 1e-6 token units at 6 decimals.
        Float oneBaseUnit = LibDecimalFloat.packLossless(1, -6);

        int256 harnessStart = int256(token.balanceOf(address(harness)));
        // netRequested is tracked in tenths of a base unit (each move is units/10
        // base units) to stay in exact integer arithmetic; compared as a Float.
        int256 netRequestedTenths = 0;

        for (uint256 i = 0; i < n; i++) {
            // Each move is between 0.0 and 25.5 base units (units/10), spanning
            // sub-unit, whole, and mixed magnitudes.
            uint256 u = units[i];
            // requested amount in token units: u * 1e-7 (u tenths of a base unit).
            Float amount = LibDecimalFloat.packLossless(int256(u), -7);

            if (isPull[i]) {
                harness.exposedPull(owner, address(token), amount);
                netRequestedTenths += int256(u);
            } else {
                harness.exposedPush(owner, address(token), amount);
                netRequestedTenths -= int256(u);
            }

            // Conservation: realBalanceDelta == netRequested + credit (token units).
            int256 harnessDeltaBaseUnits = int256(token.balanceOf(address(harness))) - harnessStart;
            Float realDelta = LibDecimalFloat.packLossless(harnessDeltaBaseUnits, -6);
            Float credit = harness.exposedDustCredit(owner, address(token));
            Float netRequested = LibDecimalFloat.packLossless(netRequestedTenths, -7);
            assertTrue(realDelta.eq(netRequested.add(credit)), "conservation: realDelta == netRequested + credit");

            // Solvency: 0 <= credit < 1 base unit.
            assertTrue(!credit.lt(LibDecimalFloat.FLOAT_ZERO), "credit non-negative");
            assertTrue(credit.lt(oneBaseUnit), "credit under one base unit");
        }
    }
}
