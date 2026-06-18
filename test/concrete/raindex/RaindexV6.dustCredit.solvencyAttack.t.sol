// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";
import {Float, LibDecimalFloat} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";
import {LibRainDeploy} from "rain-deploy-0.1.2/src/lib/LibRainDeploy.sol";
import {LibTOFUTokenDecimals} from "rain-tofu-erc20-decimals-0.1.1/src/lib/LibTOFUTokenDecimals.sol";
import {
    RaindexV6DustCreditSolvencyAttackHarness
} from "test/concrete/raindex/RaindexV6DustCreditSolvencyAttackHarness.sol";

/// @title Adversarial solvency attack on the dust-credit ledger.
/// @notice Tries to construct a sequence where realizing a credit pays out
/// tokens the orderbook does not hold for that user (draining a pooled balance /
/// a third party), or where a credit is backed by nothing.
contract RaindexV6DustCreditSolvencyAttackTest is Test {
    using LibDecimalFloat for Float;

    RaindexV6DustCreditSolvencyAttackHarness internal harness;
    address internal alice = makeAddr("alice");
    address internal bob = makeAddr("bob");

    function _deployHarness() internal {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);
        harness = RaindexV6DustCreditSolvencyAttackHarness(
            payable(address(uint160(uint256(keccak256("dust.credit.solvency.attack")))))
        );
        vm.etch(address(harness), type(RaindexV6DustCreditSolvencyAttackHarness).runtimeCode);
    }

    function _token(uint8 decimals) internal returns (MockToken token) {
        token = new MockToken("Token", "TKN", decimals);
        // Mint a huge but bounded supply so big transfers don't run out of
        // balance: the point of these tests is the credit math, not ERC20 limits.
        uint256 supply = type(uint256).max / 4;
        token.mint(alice, supply);
        token.mint(bob, supply);
        token.mint(address(harness), supply);
        vm.prank(alice);
        token.approve(address(harness), type(uint256).max);
        vm.prank(bob);
        token.approve(address(harness), type(uint256).max);
    }

    /// Cross-account pooled solvency. Two users churn pulls and pushes for the
    /// SAME token against ONE pooled real balance. After every move the global
    /// solvency identity must hold:
    ///   realDelta == netRequested(alice) + netRequested(bob)
    ///                + credit(alice) + credit(bob)
    /// and each credit stays in [0, 1 base unit). If two users' credits ever
    /// double-counted the same real balance, this identity would break (real
    /// would fall short of the obligations).
    /// forge-config: default.fuzz.runs = 4000
    function testCrossAccountPooledSolvency(uint8[] memory moves, uint8[] memory units) external {
        _deployHarness();
        MockToken token = _token(6);

        vm.assume(moves.length > 0 && units.length > 0);
        uint256 n = moves.length < units.length ? moves.length : units.length;

        int256 startReal = int256(token.balanceOf(address(harness)));
        // net requested per user in tenths of a base unit (exact integer math).
        int256 netAliceTenths = 0;
        int256 netBobTenths = 0;

        Float oneBaseUnit = LibDecimalFloat.packLossless(1, -6);

        for (uint256 i = 0; i < n; i++) {
            uint8 m = moves[i];
            uint256 u = units[i];
            Float amount = LibDecimalFloat.packLossless(int256(u), -7);
            // 2 low bits select account+direction: 00 alice pull, 01 alice push,
            // 10 bob pull, 11 bob push.
            address who = (m & 2) == 0 ? alice : bob;
            bool isPull = (m & 1) == 0;

            if (isPull) {
                harness.exposedPull(who, address(token), amount);
                if (who == alice) netAliceTenths += int256(u);
                else netBobTenths += int256(u);
            } else {
                harness.exposedPush(who, address(token), amount);
                if (who == alice) netAliceTenths -= int256(u);
                else netBobTenths -= int256(u);
            }

            _checkPooled(token, startReal, netAliceTenths, netBobTenths, oneBaseUnit);
        }
    }

    function _checkPooled(
        MockToken token,
        int256 startReal,
        int256 netAliceTenths,
        int256 netBobTenths,
        Float oneBaseUnit
    ) internal view {
        int256 realDeltaBaseUnits = int256(token.balanceOf(address(harness))) - startReal;
        Float realDelta = LibDecimalFloat.packLossless(realDeltaBaseUnits, -6);

        Float creditAlice = harness.exposedDustCredit(alice, address(token));
        Float creditBob = harness.exposedDustCredit(bob, address(token));

        Float netAlice = LibDecimalFloat.packLossless(netAliceTenths, -7);
        Float netBob = LibDecimalFloat.packLossless(netBobTenths, -7);

        Float expected = netAlice.add(netBob).add(creditAlice).add(creditBob);
        assertTrue(realDelta.eq(expected), "global pooled solvency identity");

        // Solvency bounds on each per-user credit.
        assertTrue(!creditAlice.lt(LibDecimalFloat.FLOAT_ZERO), "alice credit >= 0");
        assertTrue(creditAlice.lt(oneBaseUnit), "alice credit < 1 base unit");
        assertTrue(!creditBob.lt(LibDecimalFloat.FLOAT_ZERO), "bob credit >= 0");
        assertTrue(creditBob.lt(oneBaseUnit), "bob credit < 1 base unit");
    }

    /// Same-token same-tx arrive-and-leave, mirroring clear3: token X is pulled
    /// from alice (over-pull, +credit alice) and pushed to bob (under-push,
    /// +credit bob) in the same logical clear. Verify neither credit is backed by
    /// the other's tokens: realDelta must equal netRequested + both credits.
    function testArriveAndLeaveSameTxSameToken() external {
        _deployHarness();
        MockToken token = _token(6);

        int256 startReal = int256(token.balanceOf(address(harness)));

        // alice's vault-0 output of 1.5 base units -> pulled, rounds up to 2.
        Float aliceOut = LibDecimalFloat.packLossless(15, -7);
        // bob's vault-0 input of 1.5 base units -> pushed, rounds down to 1.
        Float bobIn = LibDecimalFloat.packLossless(15, -7);

        // clear3 ordering: pull outputs first, then push inputs.
        (uint256 pulled,) = harness.exposedPull(alice, address(token), aliceOut);
        (uint256 pushed,) = harness.exposedPush(bob, address(token), bobIn);

        assertEq(pulled, 2, "alice over-pull rounds up");
        assertEq(pushed, 1, "bob under-push rounds down");

        int256 realDelta = int256(token.balanceOf(address(harness))) - startReal;
        // Net real: +2 (from alice) -1 (to bob) = +1 base unit.
        assertEq(realDelta, 1, "net real +1");

        Float creditAlice = harness.exposedDustCredit(alice, address(token));
        Float creditBob = harness.exposedDustCredit(bob, address(token));
        // alice credit 0.5 (over-pull), bob credit 0.5 (under-push).
        assertTrue(creditAlice.eq(LibDecimalFloat.packLossless(5, -7)), "alice credit 0.5");
        assertTrue(creditBob.eq(LibDecimalFloat.packLossless(5, -7)), "bob credit 0.5");

        // Global identity: realDelta(token units) == netRequested + credits.
        // netRequested = +1.5 (alice pull) - 1.5 (bob push) = 0.
        Float realDeltaF = LibDecimalFloat.packLossless(realDelta, -6);
        Float expected = creditAlice.add(creditBob); // net requested == 0
        assertTrue(realDeltaF.eq(expected), "arrive-and-leave solvency identity");
    }

    /// Large-amount precision attack on PUSH. A huge push amount whose integer
    /// transfer exceeds ~67 significant digits could make `pushed` (the Float
    /// round-trip of the integer) smaller than the integer actually sent,
    /// inflating the recorded credit beyond what was withheld -> a credit backed
    /// by nothing. We push amounts spanning the full uint256 magnitude range
    /// (mantissa * 10^expDigits base units) and assert the recorded credit equals
    /// the TRUE withheld dust (requestedTokenUnits - sentTokenUnits), and stays in
    /// [0, 1 base unit). If the lossy round-trip inflated the credit, this breaks.
    /// forge-config: default.fuzz.runs = 5000
    function testLargeAmountPushCreditNotInflated(uint256 mantissa, uint8 expDigits, uint8 decimals) external {
        decimals = uint8(bound(decimals, 0, 36));
        // mantissa fits in int224 (<= ~66 digits) so our own assertion-side
        // packLossless never overflows; expDigits supplies the extra magnitude.
        mantissa = bound(mantissa, 1, uint256(uint224(type(int224).max)));
        // Keep mantissa * 10^expDigits within the minted supply (uint256.max/4).
        expDigits = uint8(bound(expDigits, 0, 70));
        _deployHarness();
        MockToken token = _token(decimals);

        Float oneBaseUnit = LibDecimalFloat.packLossless(1, -int256(uint256(decimals)));

        // value = mantissa * 10^expDigits base units. Ensure it fits in supply.
        uint256 maxMantissa = (type(uint256).max / 4) / (10 ** uint256(expDigits));
        if (maxMantissa == 0) return;
        if (mantissa > maxMantissa) mantissa = maxMantissa;
        if (mantissa == 0) return;

        // requested in TOKEN units = mantissa * 10^(expDigits - decimals).
        Float requested =
            LibDecimalFloat.packLossless(int256(mantissa), int256(uint256(expDigits)) - int256(uint256(decimals)));

        uint256 balBefore = token.balanceOf(address(harness));
        (uint256 sent,) = harness.exposedPush(alice, address(token), requested);

        // The orderbook never sends out more than it held.
        assertLe(sent, balBefore, "never sends more than held");

        Float credit = harness.exposedDustCredit(alice, address(token));
        assertTrue(!credit.lt(LibDecimalFloat.FLOAT_ZERO), "post-big credit >= 0");
        assertTrue(credit.lt(oneBaseUnit), "post-big credit < 1 base unit");

        // Core solvency check: the recorded credit must not exceed the TRUE
        // withheld dust = requested - sent (token units). If a lossy round-trip
        // inflated `credit` above the real shortfall, the orderbook would be
        // recording a debt it never withheld -> a credit backed by nothing.
        // `sent` can exceed int224; pack it lossily and use >= so any inflation
        // of `credit` beyond the real (possibly under-counted) shortfall trips.
        (Float sentTokenUnits,) = LibDecimalFloat.fromFixedDecimalLossyPacked(sent, decimals);
        Float trueWithheld = requested.sub(sentTokenUnits);
        // credit must be <= trueWithheld (never inflated above what was withheld).
        assertTrue(!credit.gt(trueWithheld), "credit <= true withheld dust (not inflated)");
    }

    /// Direct seed-then-drain attack: can an accumulated push credit ever pay out
    /// tokens that were not actually withheld? Build up credit to just under a
    /// base unit, then realize it, and confirm the realized base unit was indeed
    /// covered by tokens previously NOT sent (i.e. total sent over the whole
    /// sequence equals floor(total requested + 0) exactly, never more).
    function testPushNeverPaysMoreThanRequestedPlusWithheld() external {
        _deployHarness();
        MockToken token = _token(6);

        // Request 0.9 base units nine times = 8.1 base units total requested.
        // floor accounting: cumulative sent must always equal
        // floor(cumulativeRequested) and never exceed it.
        Float amount = LibDecimalFloat.packLossless(9, -7); // 0.9 base units
        uint256 totalSent = 0;
        // requested cumulative in tenths of a base unit.
        uint256 reqTenths = 0;
        for (uint256 k = 0; k < 12; k++) {
            (uint256 sent,) = harness.exposedPush(alice, address(token), amount);
            totalSent += sent;
            reqTenths += 9;
            // Invariant: never sent more than the whole-base-units requested.
            assertLe(totalSent, reqTenths / 10, "never over-pays cumulatively");
            // And the shortfall is exactly the standing credit.
            Float credit = harness.exposedDustCredit(alice, address(token));
            // requested - sent (token units) must equal credit.
            Float requested = LibDecimalFloat.packLossless(int256(reqTenths), -7);
            Float sentF = LibDecimalFloat.packLossless(int256(totalSent), -6);
            assertTrue(requested.sub(sentF).eq(credit), "withheld == credit exactly");
        }
    }
}
