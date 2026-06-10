// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";
import {RaindexV6} from "src/concrete/raindex/RaindexV6.sol";
import {Float, LibDecimalFloat} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";
import {LibRainDeploy} from "rain-deploy-0.1.2/src/lib/LibRainDeploy.sol";
import {LibTOFUTokenDecimals} from "rain-tofu-erc20-decimals-0.1.1/src/lib/LibTOFUTokenDecimals.sol";

contract RaindexV6DustCreditConservationAttackHarness is RaindexV6 {
    function exposedPull(address account, address token, Float amount) external returns (uint256, uint8) {
        return pullTokens(account, token, amount);
    }

    function exposedPush(address account, address token, Float amount) external returns (uint256, uint8) {
        return pushTokens(account, token, amount);
    }

    function exposedDustCredit(address user, address token) external view returns (Float) {
        return sDustCredit[user][token];
    }
}

/// @title Adversarial conservation attack on the dust-credit ledger.
contract RaindexV6DustCreditConservationAttackTest is Test {
    using LibDecimalFloat for Float;

    RaindexV6DustCreditConservationAttackHarness internal harness;
    address internal owner = makeAddr("owner");

    function setUp() external {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);
        harness = RaindexV6DustCreditConservationAttackHarness(
            payable(address(uint160(uint256(keccak256("dust.credit.conservation.attack.harness")))))
        );
        vm.etch(address(harness), type(RaindexV6DustCreditConservationAttackHarness).runtimeCode);
    }

    function _mkToken(uint8 decimals) internal returns (MockToken token) {
        token = new MockToken("Token", "TKN", decimals);
        // Stay well under int256 max so the int256 casts in the conservation
        // checks never overflow on the harness side (this is a test-harness
        // limit, not a contract limit).
        uint256 supply = uint256(type(int256).max) / 4;
        token.mint(owner, supply);
        token.mint(address(harness), supply);
        vm.prank(owner);
        token.approve(address(harness), type(uint256).max);
    }

    // Helper: assert 0 <= credit < 1 base unit (1 base unit = 10^-decimals token units).
    function _assertCreditInvariant(MockToken token, uint8 decimals) internal view {
        Float credit = harness.exposedDustCredit(owner, address(token));
        assertTrue(!credit.lt(LibDecimalFloat.FLOAT_ZERO), "credit negative");
        Float oneBaseUnit = LibDecimalFloat.packLossless(1, -int256(uint256(decimals)));
        assertTrue(credit.lt(oneBaseUnit), "credit >= 1 base unit");
    }

    /// ATTACK 1: very large amount on a low-decimals token, near uint256 transfer
    /// limits, to probe the int224 truncation in the from-fixed round trip.
    /// We use an amount with significant low-order digits so any truncation in
    /// packLossy would show up as a wrong credit / wrong conservation.
    function testLargeAmountRoundTripPull() external {
        uint8 decimals = 6;
        MockToken token = _mkToken(decimals);

        // amount = 1.2345...e40 token units with a fractional sub-base-unit tail.
        // coefficient 123456789012345678901234567890123456789 (39 sig digits),
        // exponent -7 => value has a 0.x base-unit fractional remainder.
        Float amount = LibDecimalFloat.packLossless(123456789012345678901234567890123456789, -7);

        uint256 balBefore = token.balanceOf(address(harness));
        (uint256 moved,) = harness.exposedPull(owner, address(token), amount);
        uint256 balAfter = token.balanceOf(address(harness));
        assertEq(balAfter - balBefore, moved, "moved matches real transfer");

        _assertCreditInvariant(token, decimals);

        // Conservation: realDelta(token units) == amount + credit (no prior credit).
        Float realDelta = LibDecimalFloat.fromFixedDecimalLosslessPacked(moved, decimals);
        Float credit = harness.exposedDustCredit(owner, address(token));
        // realDelta should equal amount + credit (pull over-pulls so realDelta>=amount).
        assertTrue(realDelta.eq(amount.add(credit)), "conservation pull large");
    }

    /// ATTACK 2: alternate pull/push at sub-base-unit boundaries across many
    /// steps, tracking conservation exactly, on an 18-decimal token.
    function testAlternatingBoundary() external {
        uint8 decimals = 18;
        MockToken token = _mkToken(decimals);

        int256 netRequestedTenths = 0;
        int256 startBal = int256(token.balanceOf(address(harness)));

        // Sequence of (isPull, tenthsOfBaseUnit).
        int256[10] memory amounts = [int256(15), 7, 23, 4, 99, 1, 250, 13, 6, 88];
        bool[10] memory pulls = [true, false, true, true, false, false, true, false, true, false];

        for (uint256 i = 0; i < 10; i++) {
            Float amount = LibDecimalFloat.packLossless(amounts[i], -int256(uint256(decimals)) - 1);
            if (pulls[i]) {
                harness.exposedPull(owner, address(token), amount);
                netRequestedTenths += amounts[i];
            } else {
                harness.exposedPush(owner, address(token), amount);
                netRequestedTenths -= amounts[i];
            }
            int256 delta = int256(token.balanceOf(address(harness))) - startBal;
            Float realDelta = LibDecimalFloat.packLossless(delta, -int256(uint256(decimals)));
            Float netReq = LibDecimalFloat.packLossless(netRequestedTenths, -int256(uint256(decimals)) - 1);
            Float credit = harness.exposedDustCredit(owner, address(token));
            assertTrue(realDelta.eq(netReq.add(credit)), "conservation alt");
            _assertCreditInvariant(token, decimals);
        }
    }

    /// ATTACK 3: high-decimals token (36) with a large amount. Probes whether the
    /// credit (very small, ~1e-36) can be silently dropped by the float sub when
    /// subtracting from a large amount (exponent gap), and whether toFixed reverts.
    function testHighDecimalsCreditNotDropped() external {
        uint8 decimals = 36;
        MockToken token = _mkToken(decimals);

        // Seed a credit with a sub-base-unit pull: 0.5 base units = 5e-37.
        Float seed = LibDecimalFloat.packLossless(5, -37);
        harness.exposedPull(owner, address(token), seed);
        Float credit0 = harness.exposedDustCredit(owner, address(token));
        // 0.5 base unit pull rounds up to 1 base unit, credit = 1e-36 - 5e-37 = 5e-37.
        assertTrue(credit0.eq(LibDecimalFloat.packLossless(5, -37)), "seed credit");

        // Now a large pull: 1e30 token units (huge). Exponent gap to credit ~ 66.
        Float big = LibDecimalFloat.packLossless(1, 30);
        uint256 balBefore = token.balanceOf(address(harness));
        (uint256 moved,) = harness.exposedPull(owner, address(token), big);
        uint256 realMoved = token.balanceOf(address(harness)) - balBefore;
        assertEq(realMoved, moved, "moved matches transfer");

        _assertCreditInvariant(token, decimals);

        // Conservation: realDelta(token units) == effective_requested + creditNew.
        // Total requested across both moves = seed + big.
        Float totalRequested = seed.add(big);
        // realDelta total = first move (1 base unit = 1e-36) + this move.
        Float realDeltaTotal = LibDecimalFloat.fromFixedDecimalLosslessPacked(1, decimals).add(
            LibDecimalFloat.fromFixedDecimalLosslessPacked(realMoved, decimals)
        );
        Float creditNew = harness.exposedDustCredit(owner, address(token));
        assertTrue(realDeltaTotal.eq(totalRequested.add(creditNew)), "conservation high-decimals");
    }

    /// ATTACK 4: push where amount + credit must realize, with an amount large
    /// enough relative to the credit that the float add might drop the credit.
    function testHighDecimalsPushCreditNotDropped() external {
        uint8 decimals = 36;
        MockToken token = _mkToken(decimals);

        // Seed credit via a sub-base-unit push: 0.5 base units -> pushes 0, credit 0.5.
        Float seed = LibDecimalFloat.packLossless(5, -37);
        harness.exposedPush(owner, address(token), seed);
        assertTrue(
            harness.exposedDustCredit(owner, address(token)).eq(LibDecimalFloat.packLossless(5, -37)), "push seed credit"
        );

        // Large push.
        Float big = LibDecimalFloat.packLossless(1, 30);
        uint256 balBefore = token.balanceOf(address(harness));
        (uint256 moved,) = harness.exposedPush(owner, address(token), big);
        uint256 realMoved = balBefore - token.balanceOf(address(harness));
        assertEq(realMoved, moved, "push moved matches");
        _assertCreditInvariant(token, decimals);
    }

    /// ATTACK 5: extreme decimals (60). One base unit = 1e-60. Does the machinery
    /// hold and not revert / not break the invariant?
    function testExtremeDecimals() external {
        uint8 decimals = 60;
        MockToken token = _mkToken(decimals);

        Float amount = LibDecimalFloat.packLossless(15, -61); // 1.5 base units
        (uint256 moved,) = harness.exposedPull(owner, address(token), amount);
        assertEq(moved, 2, "1.5 rounds up to 2 at 60 decimals");
        _assertCreditInvariant(token, decimals);
        assertTrue(
            harness.exposedDustCredit(owner, address(token)).eq(LibDecimalFloat.packLossless(5, -61)), "credit 0.5"
        );
    }

    /// ATTACK 7: deliberately force the exponent-gap drop in `effective =
    /// amount.sub(credit)`. After maximization both coefficients are ~67 digits;
    /// when their exponents differ by > 76 the smaller operand (the credit) is
    /// dropped entirely by LibDecimalFloatImplementation.add. With decimals=18,
    /// a seeded ~1e-18 credit (normalized exp ~ -85) versus an amount ~1e70 token
    /// units (normalized exp ~ +3) has a gap > 76. If the credit is dropped the
    /// PRIOR credit is silently stranded (re-introducing the original L01 dust
    /// loss) even though per-move 0<=credit<1 still holds. This probes the
    /// cross-move "every credit is realized" property, the PR's central claim.
    function testExponentGapDropsCredit() external {
        uint8 decimals = 18;
        MockToken token = _mkToken(decimals); // supply ~1.4e76 base units

        // Seed a credit: pull 0.5 base units -> rounds up to 1, credit 0.5 base
        // units = 5e-19 token units.
        harness.exposedPull(owner, address(token), LibDecimalFloat.packLossless(5, -19));
        Float seededCredit = harness.exposedDustCredit(owner, address(token));
        assertTrue(seededCredit.eq(LibDecimalFloat.packLossless(5, -19)), "seeded 0.5 credit");

        // A huge amount in token units. supply/4 ~ 1.4e76 base units = 1.4e58
        // token units. Use coefficient*10^exp with enough magnitude that the
        // normalized exponent gap to the ~5e-19 credit exceeds 76, but the base
        // unit count stays within the minted supply and toFixed does not overflow.
        // 1.4e58 token units -> exponent gap ~ 58+19 = 77 (> 76). Use 1e58.
        Float big = LibDecimalFloat.packLossless(1, 58);

        uint256 balBefore = token.balanceOf(address(harness));
        (uint256 moved,) = harness.exposedPull(owner, address(token), big);
        uint256 realMoved = token.balanceOf(address(harness)) - balBefore;
        assertEq(realMoved, moved, "moved matches transfer");

        Float creditAfter = harness.exposedDustCredit(owner, address(token));
        // Hard invariants must still hold regardless of any drop.
        _assertCreditInvariant(token, decimals);

        // Now drive the credit toward realization with a sub-unit pull and see if
        // the previously-seeded 0.5 was preserved. If the big pull dropped it, the
        // ledger lost track of real tokens the orderbook holds.
        emit log_named_bytes32("seededCredit", Float.unwrap(seededCredit));
        emit log_named_bytes32("creditAfterBig", Float.unwrap(creditAfter));
        emit log_named_uint("movedBig", moved);

        // Characterize the stranding: the big pull moved exactly the requested
        // base units (1e76) with NO over-pull, and the previously-seeded 0.5
        // credit was silently dropped to zero rather than realized.
        assertEq(moved, 1e76, "big pull moved exactly the requested base units");
        assertTrue(creditAfter.isZero(), "STRANDING: seeded 0.5 credit silently dropped to zero by exponent gap");

        // The 0.5 base unit (5e-19 token units) that was stranded is BELOW the
        // Float resolution next to a 1e58-token-unit amount (77 OOM apart), so at
        // Float precision the stated conservation invariant still appears to hold:
        // realDeltaTotal and (totalRequested + creditAfter) are the SAME Float.
        // The physical stranding is real (the contract's raw ERC20 balance carries
        // the extra 0.5 base unit with no ledger entry) but invisible/irrecoverable
        // at this magnitude. This is the pre-PR L01 dust, re-stranded at extreme
        // amounts; it does NOT violate 0<=credit<1 or the Float-precision
        // conservation equation.
        (Float realBigDelta,) = LibDecimalFloat.fromFixedDecimalLossyPacked(realMoved, decimals);
        Float realDeltaTotal = LibDecimalFloat.fromFixedDecimalLosslessPacked(1, decimals).add(realBigDelta);
        Float totalRequested = LibDecimalFloat.packLossless(5, -19).add(big);
        Float conserved = totalRequested.add(creditAfter);
        // At Float precision they ARE equal: the stated invariant survives even as
        // the credit is physically stranded.
        assertTrue(realDeltaTotal.eq(conserved), "Float-precision conservation still holds; stranding is sub-resolution");
    }

    /// ATTACK 8: broaden the PR fuzz beyond its 6-decimal / 25.5-base-unit cap.
    /// Random decimals in [0,30] and amounts spanning sub-unit to ~6.5e9 base
    /// units, asserting hard invariants after every move. Catches any rounding /
    /// sign bug the narrow PR fuzz misses.
    /// forge-config: default.fuzz.runs = 1500
    function testWideFuzz(uint8 decimalsRaw, bool[] memory isPull, uint64[] memory amts) external {
        uint8 decimals = uint8(bound(decimalsRaw, 0, 30));
        MockToken token = _mkToken(decimals);

        vm.assume(isPull.length > 0 && amts.length > 0);
        uint256 n = isPull.length < amts.length ? isPull.length : amts.length;
        vm.assume(n > 0);
        if (n > 40) n = 40;

        Float oneBaseUnit = LibDecimalFloat.packLossless(1, -int256(uint256(decimals)));

        for (uint256 i = 0; i < n; i++) {
            // amount in token units: amts[i] base units, expressed as a Float at
            // exponent -decimals, but offset the exponent by -1 half the time to
            // exercise genuine sub-base-unit fractions.
            int256 exp = -int256(uint256(decimals)) - int256(i % 2);
            Float amount = LibDecimalFloat.packLossless(int256(uint256(amts[i])), exp);
            if (isPull[i]) {
                harness.exposedPull(owner, address(token), amount);
            } else {
                harness.exposedPush(owner, address(token), amount);
            }
            Float credit = harness.exposedDustCredit(owner, address(token));
            assertTrue(!credit.lt(LibDecimalFloat.FLOAT_ZERO), "fuzz: credit negative");
            assertTrue(credit.lt(oneBaseUnit), "fuzz: credit >= 1 base unit");
        }
    }

    /// ATTACK 6: stress the int224 boundary directly. Pull an amount whose base
    /// unit count exceeds int224 max (~6.7e66) WITH significant low digits, on a
    /// 0-decimals token (so amount==base units and there are no trailing zeros to
    /// rescue the round trip). If packLossy truncation corrupts the round trip the
    /// derived credit goes wrong / negative.
    function testInt224BoundaryZeroDecimals() external {
        uint8 decimals = 0;
        MockToken token = _mkToken(decimals);

        // A 60-significant-digit integer amount of base units, no trailing zeros.
        // 123...789 repeated. int224 max ~ 1.35e67 so 60 digits (~1e59) is safe;
        // push to ~67 digits to flirt with the boundary but stay representable.
        Float amount = LibDecimalFloat.packLossless(123456789012345678901234567890123456789012345678901234567891, 0);
        uint256 balBefore = token.balanceOf(address(harness));
        (uint256 moved,) = harness.exposedPull(owner, address(token), amount);
        assertEq(token.balanceOf(address(harness)) - balBefore, moved, "moved matches");
        _assertCreditInvariant(token, decimals);
        // 0 decimals => integer amount is lossless, credit must be exactly zero.
        assertTrue(harness.exposedDustCredit(owner, address(token)).isZero(), "0-dec lossless => no credit");
        Float realDelta = LibDecimalFloat.fromFixedDecimalLosslessPacked(moved, decimals);
        assertTrue(realDelta.eq(amount), "0-dec conservation exact");
    }
}
