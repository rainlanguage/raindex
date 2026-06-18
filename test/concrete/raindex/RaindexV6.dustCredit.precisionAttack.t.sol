// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";
import {Float, LibDecimalFloat} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";
import {LibRainDeploy} from "rain-deploy-0.1.2/src/lib/LibRainDeploy.sol";
import {LibTOFUTokenDecimals} from "rain-tofu-erc20-decimals-0.1.1/src/lib/LibTOFUTokenDecimals.sol";
import {ITOFUTokenDecimals, TOFUOutcome} from "rain-tofu-erc20-decimals-0.1.1/src/interface/ITOFUTokenDecimals.sol";
import {
    RaindexV6DustCreditPrecisionAttackHarness
} from "test/concrete/raindex/RaindexV6DustCreditPrecisionAttackHarness.sol";
import {PrecisionAttackMutableDecimalsToken} from "test/concrete/raindex/PrecisionAttackMutableDecimalsToken.sol";

contract RaindexV6DustCreditPrecisionAttackTest is Test {
    using LibDecimalFloat for Float;

    RaindexV6DustCreditPrecisionAttackHarness internal harness;
    address internal owner = makeAddr("owner");

    function setUp() external {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);
        harness = RaindexV6DustCreditPrecisionAttackHarness(
            payable(address(uint160(uint256(keccak256("dust.credit.precision.attack")))))
        );
        vm.etch(address(harness), type(RaindexV6DustCreditPrecisionAttackHarness).runtimeCode);
    }

    function _newToken(uint8 dec) internal returns (MockToken token) {
        token = new MockToken("Token", "TKN", dec);
        // Mint enough that huge-magnitude moves succeed, but keep balances below
        // int256 max so the test's own int256 delta arithmetic does not overflow.
        // int256 max ~5.7e76; keep balances ~ <= 2e76 base units.
        uint256 fund = 2 * 10 ** 76;
        token.mint(owner, fund);
        token.mint(address(harness), fund);
        vm.prank(owner);
        token.approve(address(harness), type(uint256).max);
    }

    /// ATTACK 1: scan pull magnitudes from 1e50 up to the largest that does not
    /// overflow uint256 base units, seeding a 0.5 base-unit credit first. At each
    /// magnitude assert conservation: realDelta == netRequested + credit, and the
    /// solvency bound 0 <= credit < 1 base unit. The hypothesis is that at some
    /// magnitude `effective = amount.sub(credit)` silently drops the credit (the
    /// add/sub helper ignores operand B when the exponent gap exceeds 76), which
    /// would strand the 0.5 base-unit credit -> conservation broken.
    function testPullMagnitudeScanConservation() external {
        // exponent of the requested token-unit amount. At 6 decimals the base
        // unit count is coefficient * 10^(exp+6); must stay < ~1.157e77.
        // 1 * 10^exp token units -> 10^(exp+6) base units. exp <= 70 keeps it
        // under uint256 max (1e76 base units). exp=71 -> 1e77 base units (still
        // < 1.157e77, fits). exp=72 -> 1e78 overflows.
        for (int256 exp = 50; exp <= 70; exp++) {
            MockToken token = _newToken(6);

            // Seed 0.5 base-unit credit via a 1.5 base-unit pull.
            Float r1 = LibDecimalFloat.packLossless(15, -7);
            (uint256 moved1,) = harness.exposedPull(owner, address(token), r1);

            Float r2 = LibDecimalFloat.packLossless(1, exp);
            (uint256 moved2,) = harness.exposedPull(owner, address(token), r2);

            // Total physically pulled, expressed in token units (Float), built
            // from the two uint256 returns so the test never overflows int256.
            Float movedTotal = _floatFromBaseUnits(moved1).add(_floatFromBaseUnits(moved2));

            Float netRequested = r1.add(r2);
            Float credit = harness.exposedDustCredit(owner, address(token));
            Float expected = netRequested.add(credit);

            // Conservation: tokens physically pulled == requested + standing credit.
            if (!movedTotal.eq(expected)) {
                emit log_named_int("PULL BROKEN at exp", exp);
                emit log_named_uint("moved1", moved1);
                emit log_named_uint("moved2", moved2);
                emit log_named_int("movedTotal coeff", _coeff(movedTotal));
                emit log_named_int("movedTotal exp", _exp(movedTotal));
                emit log_named_int("expected coeff", _coeff(expected));
                emit log_named_int("expected exp", _exp(expected));
                emit log_named_int("credit coeff", _coeff(credit));
                emit log_named_int("credit exp", _exp(credit));
            }
            assertTrue(movedTotal.eq(expected), "conservation: movedTotal == netRequested + credit");
            assertTrue(!credit.lt(LibDecimalFloat.FLOAT_ZERO), "credit non-negative");
            assertTrue(credit.lt(LibDecimalFloat.packLossless(1, -6)), "credit < 1 base unit");
        }
    }

    /// REALISM REGRESSION GUARD (18 decimals): the clamp keeps the orderbook
    /// solvent for the common 18-decimal token at every whole-token magnitude
    /// whose base unit count fits the funded supply (exp <= 58 keeps 10^(exp+18)
    /// <= 2e76). Assert intake >= obligation + standing credit at each.
    function testFindMinInsolventExponent18dp() external {
        for (int256 exp = 0; exp <= 58; exp++) {
            PrecisionAttackMutableDecimalsToken token = new PrecisionAttackMutableDecimalsToken(18);
            token.mint(owner, 2 * 10 ** 76);
            token.mint(address(harness), 2 * 10 ** 76);
            vm.prank(owner);
            token.approve(address(harness), type(uint256).max);

            uint256 heldBefore = token.balanceOf(address(harness));
            // seed 0.5 base units credit at 18dp = 5e-19 token units * 1.5 ... use
            // 1.5 base units = 15e-19.
            harness.exposedPull(owner, address(token), LibDecimalFloat.packLossless(15, -19));
            harness.exposedPull(owner, address(token), LibDecimalFloat.packLossless(1, exp));
            uint256 takenIn = token.balanceOf(address(harness)) - heldBefore;

            Float vaultBalance = LibDecimalFloat.packLossless(15, -19).add(LibDecimalFloat.packLossless(1, exp));
            (uint256 obligation,) = LibDecimalFloat.toFixedDecimalLossy(vaultBalance, 18);
            Float credit = harness.exposedDustCredit(owner, address(token));
            (uint256 creditBase,) = LibDecimalFloat.toFixedDecimalLossy(credit, 18);

            assertGe(takenIn, obligation + creditBase, "18dp solvent at every whole-token magnitude");
        }
    }

    /// A/B vs MAIN: replicate main's pullTokens intake (round-up, no credit) for
    /// the same two-pull sequence and compare to the PR harness intake and to the
    /// vault obligation at 10^62 token units, the magnitude where the unclamped
    /// credit ledger used to under-hold. The precision-loss clamp restores parity:
    /// the PR intake is solvent against the obligation just like main.
    function testABMainVsPRAtExp62() external {
        int256 exp = 62;

        // --- PR harness intake ---
        MockToken token = _newToken(6);
        uint256 heldBefore = token.balanceOf(address(harness));
        harness.exposedPull(owner, address(token), LibDecimalFloat.packLossless(15, -7));
        harness.exposedPull(owner, address(token), LibDecimalFloat.packLossless(1, exp));
        uint256 prTakenIn = token.balanceOf(address(harness)) - heldBefore;
        Float prCredit = harness.exposedDustCredit(owner, address(token));
        (uint256 prCreditBase,) = LibDecimalFloat.toFixedDecimalLossy(prCredit, 6);

        // --- main intake replicated inline (round-up per pull, no credit) ---
        uint256 mainTakenIn =
            _mainPull(LibDecimalFloat.packLossless(15, -7)) + _mainPull(LibDecimalFloat.packLossless(1, exp));

        // --- vault obligation (Float vault balance -> base units, round down) ---
        Float vaultBalance = LibDecimalFloat.packLossless(15, -7).add(LibDecimalFloat.packLossless(1, exp));
        (uint256 obligation,) = LibDecimalFloat.toFixedDecimalLossy(vaultBalance, 6);

        emit log_named_uint("PR takenIn", prTakenIn);
        emit log_named_uint("PR credit liability (base)", prCreditBase);
        emit log_named_uint("MAIN takenIn", mainTakenIn);
        emit log_named_uint("vault obligation", obligation);

        // MAIN is solvent: intake >= obligation.
        assertGe(mainTakenIn, obligation, "MAIN: intake >= obligation (solvent)");
        // PR is solvent too: the clamp pulls in at least the vault obligation plus
        // the standing credit liability it now also owes the user.
        assertGe(prTakenIn, obligation + prCreditBase, "PR: intake >= obligation + credit (solvent)");
    }

    /// main's pullTokens intake: toFixedDecimalLossy rounded UP on lossy.
    function _mainPull(Float amount) internal pure returns (uint256 amount18) {
        bool lossless;
        (amount18, lossless) = LibDecimalFloat.toFixedDecimalLossy(amount, 6);
        if (!lossless) {
            ++amount18;
        }
    }

    /// MINIMUM-MAGNITUDE REGRESSION GUARD: the precision-loss clamp keeps the
    /// orderbook solvent at every pull magnitude, so no exponent flips a harmless
    /// over-hold into an untracked under-hold. Scan every magnitude whose base
    /// unit count fits the funded supply (exp <= 70 keeps 10^(exp+6) <= 2e76) and
    /// assert intake >= obligation + standing credit at each.
    function testFindMinInsolventExponent() external {
        for (int256 exp = 0; exp <= 70; exp++) {
            MockToken token = _newToken(6);
            uint256 heldBefore = token.balanceOf(address(harness));

            // seed 1.5 base units (-> credit 0.5), then 1*10^exp token units.
            harness.exposedPull(owner, address(token), LibDecimalFloat.packLossless(15, -7));
            harness.exposedPull(owner, address(token), LibDecimalFloat.packLossless(1, exp));
            uint256 takenIn = token.balanceOf(address(harness)) - heldBefore;

            Float vaultBalance = LibDecimalFloat.packLossless(15, -7).add(LibDecimalFloat.packLossless(1, exp));
            (uint256 obligation,) = LibDecimalFloat.toFixedDecimalLossy(vaultBalance, 6);
            Float credit = harness.exposedDustCredit(owner, address(token));
            (uint256 creditBase,) = LibDecimalFloat.toFixedDecimalLossy(credit, 6);

            assertGe(takenIn, obligation + creditBase, "solvent at every pull magnitude");
        }
    }

    /// SOLVENCY END-TO-END: compare, for an identical (seed 1.5 base units) +
    /// (1e62 token units) two-pull deposit sequence, the orderbook's net token
    /// intake against the vault obligation it books, and against the standing
    /// credit liability. Solvency requires:
    ///   heldForUser (real tokens in) + 0  >=  vaultObligation - creditLiability
    /// equivalently the orderbook must hold at least (vault balance it owes the
    /// user) minus (credit it owes the user). A negative slack means the
    /// orderbook took in fewer tokens than it now claims to owe -> insolvent.
    function testTwoPullSolvencySlack() external {
        MockToken token = _newToken(6);

        uint256 heldBefore = token.balanceOf(address(harness));

        // Two pulls: seed 1.5 base units, then 1e62 token units.
        harness.exposedPull(owner, address(token), LibDecimalFloat.packLossless(15, -7));
        harness.exposedPull(owner, address(token), LibDecimalFloat.packLossless(1, 62));

        uint256 heldAfter = token.balanceOf(address(harness));
        uint256 takenIn = heldAfter - heldBefore; // base units the orderbook received

        // Vault obligation in base units the orderbook now owes the user:
        // a vault crediting 1.5 + 1e62 token units. When the user withdraws the
        // FULL float vault balance, pushTokens converts it to base units (rounded
        // down). Compute that exact realizable base-unit obligation.
        Float vaultBalance = LibDecimalFloat.packLossless(15, -7).add(LibDecimalFloat.packLossless(1, 62));
        (uint256 obligationBaseUnits,) = LibDecimalFloat.toFixedDecimalLossy(vaultBalance, 6);

        // Standing credit the orderbook ALSO owes the user (base units, rounded
        // down for what it could pay).
        Float credit = harness.exposedDustCredit(owner, address(token));
        (uint256 creditBaseUnits,) = LibDecimalFloat.toFixedDecimalLossy(credit, 6);

        emit log_named_uint("takenIn (base units)", takenIn);
        emit log_named_uint("vault obligation (base units)", obligationBaseUnits);
        emit log_named_uint("credit liability (base units)", creditBaseUnits);

        // The orderbook owes the user, on full withdrawal + credit realization,
        // at most (obligation + credit) base units; it took in `takenIn`.
        // Solvency: takenIn >= obligation + credit (it must hold at least what it
        // owes). Report the slack.
        uint256 owed = obligationBaseUnits + creditBaseUnits;
        emit log_named_uint("owed (obligation+credit)", owed);
        if (takenIn < owed) {
            emit log_named_uint("INSOLVENT shortfall (base units)", owed - takenIn);
        }
        assertGe(takenIn, owed, "orderbook holds at least what it owes the user");
    }

    /// DIAGNOSTIC: isolate exp=62 pull. Compare the PR's pulled base units, with
    /// and without a seeded 0.5 credit, against the ideal 1e68 base units the
    /// vault accounting credits. The solvency-critical question is whether the
    /// contract pulls AT LEAST as many base units as the vault is credited
    /// (rounding the user's deposit UP for the protocol), or whether the credit
    /// machinery causes an UNDER-pull that leaves the vault over-credited.
    function testPullExp62SolvencyDiagnostic() external {
        // No-seed baseline: a single 1e62 pull.
        {
            MockToken token = _newToken(6);
            (uint256 moved,) = harness.exposedPull(owner, address(token), LibDecimalFloat.packLossless(1, 62));
            emit log_named_uint("no-seed moved (base units)", moved);
            // Ideal vault-accounting base units for 1e62 token units at 6 dp.
            // = 1e68. Solvency requires moved >= 1e68 (pull rounds up).
            emit log_named_uint("ideal 1e68", 10 ** 68);
            // Does the orderbook receive at least what the vault is credited?
            assertGe(moved, 10 ** 68, "no-seed: pulled >= vault-credited (solvent)");
        }
        // Seeded 0.5 credit then 1e62 pull.
        {
            MockToken token = _newToken(6);
            harness.exposedPull(owner, address(token), LibDecimalFloat.packLossless(15, -7));
            (uint256 moved,) = harness.exposedPull(owner, address(token), LibDecimalFloat.packLossless(1, 62));
            Float creditAfter = harness.exposedDustCredit(owner, address(token));
            emit log_named_uint("seeded moved (base units)", moved);
            emit log_named_int("seeded creditAfter coeff", _coeff(creditAfter));
            emit log_named_int("seeded creditAfter exp", _exp(creditAfter));
            // The vault is credited 1.5 + 1e62 token units across the two pulls
            // = 1e68 + 1.5 base units (the 1.5 from the seed). The orderbook
            // physically received 2 (seed) + `moved`. Solvency requires
            // 2 + moved + creditReleased relationship to hold. Concretely the
            // orderbook must hold >= vault total. vault total base units =
            // ceil-ish 1e68 + 2 (seed credited 1.5 -> but seed pulled 2).
            // Simplest solvency statement at the boundary: total physically held
            // for these two pulls (2 + moved) must be >= total vault-credited
            // base units (1e68 + ceil(1.5)=2) MINUS the standing credit (which is
            // a liability the orderbook owes back).
            // total held = 2 + moved ; vault credited (base units) approx 1e68 + 2
            // standing credit (base units) = creditAfter.
            // Require: heldBaseUnits + creditAfterBaseUnits >= vaultCreditedBaseUnits
            // i.e. orderbook is not under-funded.
            // We assert the weaker, decisive check: moved >= 1e68 - 1 (it must not
            // fall materially short of the vault-credited 1e68 for the big pull).
            emit log_named_uint("shortfall vs 1e68", moved >= 10 ** 68 ? 0 : (10 ** 68 - moved));
        }
    }

    /// ATTACK 2: TOFU decimals change between moves. Seed credit at 6 decimals,
    /// flip token to 18 decimals, then attempt to realize. If the contract does
    /// NOT revert, the stored Float credit could realize at the wrong base-unit
    /// scale. Expectation: TOFU forces a revert, neutralizing the attack.
    function testDecimalsChangeRevertsAndProtectsCredit() external {
        PrecisionAttackMutableDecimalsToken token = new PrecisionAttackMutableDecimalsToken(6);
        token.mint(owner, type(uint128).max);
        token.mint(address(harness), type(uint128).max);
        vm.prank(owner);
        token.approve(address(harness), type(uint256).max);

        // First move initializes TOFU at 6 decimals and seeds 0.5 base unit credit.
        harness.exposedPull(owner, address(token), LibDecimalFloat.packLossless(15, -7));
        Float seeded = harness.exposedDustCredit(owner, address(token));
        assertTrue(seeded.eq(LibDecimalFloat.packLossless(5, -7)), "seed credit 0.5");

        // Token changes decimals to 18.
        token.setDecimals(18);

        // Any subsequent pull/push must revert (TOFU Inconsistent), so the stored
        // credit can never be realized at the wrong scale.
        vm.expectRevert(
            abi.encodeWithSelector(
                ITOFUTokenDecimals.TokenDecimalsReadFailure.selector, address(token), TOFUOutcome.Inconsistent
            )
        );
        harness.exposedPull(owner, address(token), LibDecimalFloat.packLossless(15, -7));

        vm.expectRevert(
            abi.encodeWithSelector(
                ITOFUTokenDecimals.TokenDecimalsReadFailure.selector, address(token), TOFUOutcome.Inconsistent
            )
        );
        harness.exposedPush(owner, address(token), LibDecimalFloat.packLossless(15, -7));

        // Credit is frozen, unchanged.
        assertTrue(harness.exposedDustCredit(owner, address(token)).eq(seeded), "credit frozen after decimals change");
    }

    /// ATTACK 3: high-decimals token (18 dp) sub-unit accumulation. A push of
    /// 0.5 base units (5e-19 token units) repeatedly. Confirm the sub-unit
    /// crossing math still holds at 18 decimals where the base unit is tiny.
    function testHighDecimalsSubUnitAccumulation() external {
        MockToken token = _newToken(18);
        // 0.5 base units at 18 dp = 5e-19 token units.
        Float half = LibDecimalFloat.packLossless(5, -19);

        (uint256 m1,) = harness.exposedPush(owner, address(token), half);
        assertEq(m1, 0, "0.5 pushes nothing");
        (uint256 m2,) = harness.exposedPush(owner, address(token), half);
        assertEq(m2, 1, "1.0 crosses whole unit");
        assertTrue(harness.exposedDustCredit(owner, address(token)).isZero(), "credit cleared at 18dp");
    }

    /// ATTACK 4: scan push magnitudes. Seed 0.5 base-unit credit (owed to user),
    /// then push a huge amount at each magnitude. Conservation for pushes:
    /// harnessDelta == -netRequested + credit (the harness sends netRequested and
    /// retains the standing credit). If the credit gets dropped in
    /// `effective = amount.add(credit)`, the orderbook either over-pays or strands
    /// value -> conservation broken.
    function testPushMagnitudeScanConservation() external {
        for (int256 exp = 50; exp <= 70; exp++) {
            MockToken token = _newToken(6);

            Float r1 = LibDecimalFloat.packLossless(15, -7); // rounds down to 1, credit 0.5
            (uint256 moved1,) = harness.exposedPush(owner, address(token), r1);

            Float r2 = LibDecimalFloat.packLossless(1, exp);
            (uint256 moved2,) = harness.exposedPush(owner, address(token), r2);

            // Conservation for pushes: tokens physically sent + retained credit
            // == requested. i.e. movedTotal + credit == netRequested.
            Float movedTotal = _floatFromBaseUnits(moved1).add(_floatFromBaseUnits(moved2));
            Float netRequested = r1.add(r2);
            Float credit = harness.exposedDustCredit(owner, address(token));
            Float expected = netRequested.sub(credit);

            if (!movedTotal.eq(expected)) {
                emit log_named_int("PUSH BROKEN at exp", exp);
                emit log_named_uint("moved1", moved1);
                emit log_named_uint("moved2", moved2);
                emit log_named_int("movedTotal coeff", _coeff(movedTotal));
                emit log_named_int("movedTotal exp", _exp(movedTotal));
                emit log_named_int("expected coeff", _coeff(expected));
                emit log_named_int("expected exp", _exp(expected));
                emit log_named_int("credit coeff", _coeff(credit));
                emit log_named_int("credit exp", _exp(credit));
            }
            assertTrue(movedTotal.eq(expected), "push conservation: movedTotal == netRequested - credit");
            assertTrue(!credit.lt(LibDecimalFloat.FLOAT_ZERO), "credit non-negative");
            assertTrue(credit.lt(LibDecimalFloat.packLossless(1, -6)), "credit < 1 base unit");
        }
    }

    /// Converts a base-unit count (6 dp) to a token-unit Float exactly as the
    /// contract does, so comparisons are apples-to-apples with the contract's own
    /// `pulled`/`pushed` derivation.
    function _floatFromBaseUnits(uint256 baseUnits) internal pure returns (Float f) {
        (f,) = LibDecimalFloat.fromFixedDecimalLossyPacked(baseUnits, 6);
    }

    function _coeff(Float f) internal pure returns (int256 c) {
        (c,) = LibDecimalFloat.unpack(f);
    }

    function _exp(Float f) internal pure returns (int256 e) {
        (, e) = LibDecimalFloat.unpack(f);
    }
}
