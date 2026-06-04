// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";
import {LibTakeOrdersOverlay} from "src/lib/LibTakeOrdersOverlay.sol";
import {OrderIOCalculationV4} from "src/concrete/raindex/RaindexV6.sol";

/// @title LibTakeOrdersOverlayTest
/// @notice Direct coverage for `latestOwnerKvs`: the backward scan that picks
/// the most recent same-owner calculate kvs to seed the next order's overlay.
contract LibTakeOrdersOverlayTest is Test {
    /// Build a calculation with only the owner + kvs set (all other fields
    /// default), mirroring how `takeOrders4` records evaluated orders.
    function _calc(address owner, bytes32[] memory kvs) internal pure returns (OrderIOCalculationV4 memory calc) {
        calc.order.owner = owner;
        calc.kvs = kvs;
    }

    function _kvs(bytes32 v) internal pure returns (bytes32[] memory kvs) {
        kvs = new bytes32[](1);
        kvs[0] = v;
    }

    /// An empty accumulator yields an empty overlay.
    function testEmptyArray() external {
        OrderIOCalculationV4[] memory calcs = new OrderIOCalculationV4[](0);
        assertEq(LibTakeOrdersOverlay.latestOwnerKvs(calcs, address(0x1234)).length, 0);
    }

    /// No entry matches the owner -> empty overlay.
    function testNoMatch() external {
        OrderIOCalculationV4[] memory calcs = new OrderIOCalculationV4[](3);
        calcs[0] = _calc(address(1), _kvs(bytes32(uint256(10))));
        calcs[1] = _calc(address(2), _kvs(bytes32(uint256(20))));
        calcs[2] = _calc(address(3), _kvs(bytes32(uint256(30))));
        assertEq(LibTakeOrdersOverlay.latestOwnerKvs(calcs, address(99)).length, 0);
    }

    /// A single matching entry returns exactly that entry's kvs, by reference
    /// (not a copy) so no concatenation/allocation happens on the hot path.
    function testSingleMatchReturnsReference() external {
        OrderIOCalculationV4[] memory calcs = new OrderIOCalculationV4[](1);
        calcs[0] = _calc(address(7), _kvs(bytes32(uint256(42))));

        bytes32[] memory got = LibTakeOrdersOverlay.latestOwnerKvs(calcs, address(7));
        assertEq(got.length, 1);
        assertEq(got[0], bytes32(uint256(42)));

        // Returned by reference, not copied: mutating the result is visible in
        // the recorded entry (they alias the same memory).
        got[0] = bytes32(uint256(999));
        assertEq(calcs[0].kvs[0], bytes32(uint256(999)), "result must alias the recorded kvs (no copy)");
    }

    /// Multiple same-owner entries -> the most recent (highest index) wins,
    /// because each eval's returned kvs already merges all earlier same-owner
    /// writes.
    function testLatestOfMultipleSameOwner() external {
        OrderIOCalculationV4[] memory calcs = new OrderIOCalculationV4[](3);
        calcs[0] = _calc(address(7), _kvs(bytes32(uint256(1))));
        calcs[1] = _calc(address(7), _kvs(bytes32(uint256(2))));
        calcs[2] = _calc(address(7), _kvs(bytes32(uint256(3))));
        assertEq(LibTakeOrdersOverlay.latestOwnerKvs(calcs, address(7))[0], bytes32(uint256(3)));
    }

    /// Interleaved owners -> the latest matching owner, never another owner's.
    function testInterleavedOwners() external {
        OrderIOCalculationV4[] memory calcs = new OrderIOCalculationV4[](5);
        calcs[0] = _calc(address(0xA), _kvs(bytes32(uint256(0xA1))));
        calcs[1] = _calc(address(0xB), _kvs(bytes32(uint256(0xB1))));
        calcs[2] = _calc(address(0xA), _kvs(bytes32(uint256(0xA2))));
        calcs[3] = _calc(address(0xB), _kvs(bytes32(uint256(0xB2))));
        calcs[4] = _calc(address(0xC), _kvs(bytes32(uint256(0xC1))));
        assertEq(LibTakeOrdersOverlay.latestOwnerKvs(calcs, address(0xA))[0], bytes32(uint256(0xA2)));
        assertEq(LibTakeOrdersOverlay.latestOwnerKvs(calcs, address(0xB))[0], bytes32(uint256(0xB2)));
        assertEq(LibTakeOrdersOverlay.latestOwnerKvs(calcs, address(0xC))[0], bytes32(uint256(0xC1)));
    }

    /// Zero-owner entries (dead orders that were never recorded, and the
    /// not-yet-evaluated tail) are skipped. Relies on `new OrderIOCalculationV4[]`
    /// zero-initializing each element's `.order.owner` to address(0).
    function testZeroOwnerGapsAndTailSkipped() external {
        OrderIOCalculationV4[] memory calcs = new OrderIOCalculationV4[](5);
        calcs[0] = _calc(address(7), _kvs(bytes32(uint256(70))));
        // calcs[1] left zero (dead-order gap).
        calcs[2] = _calc(address(8), _kvs(bytes32(uint256(80))));
        // calcs[3], calcs[4] left zero (not-yet-evaluated tail).
        assertEq(LibTakeOrdersOverlay.latestOwnerKvs(calcs, address(7))[0], bytes32(uint256(70)));
        assertEq(LibTakeOrdersOverlay.latestOwnerKvs(calcs, address(8))[0], bytes32(uint256(80)));
    }

    /// address(0) is not special-cased: querying it matches the latest
    /// zero-owner entry, whose kvs is empty. Order owners are never address(0)
    /// in practice, so this never collides with a real owner.
    function testZeroOwnerQueryReturnsEmptyGapKvs() external {
        OrderIOCalculationV4[] memory calcs = new OrderIOCalculationV4[](3);
        calcs[0] = _calc(address(7), _kvs(bytes32(uint256(70))));
        // calcs[1], calcs[2] are zero-owner gaps with empty kvs.
        assertEq(LibTakeOrdersOverlay.latestOwnerKvs(calcs, address(0)).length, 0);
    }

    /// Fuzz: the result always equals a reference backward scan over the owner
    /// layout (kvs are deterministic per index so the match is checkable).
    function testFuzzMatchesReference(address[] memory owners, address target) external {
        vm.assume(owners.length <= 64);
        OrderIOCalculationV4[] memory calcs = new OrderIOCalculationV4[](owners.length);
        for (uint256 i = 0; i < owners.length; i++) {
            calcs[i] = _calc(owners[i], _kvs(bytes32(i + 1)));
        }

        bytes32[] memory got = LibTakeOrdersOverlay.latestOwnerKvs(calcs, target);

        bool found;
        uint256 idx;
        for (uint256 i = owners.length; i > 0;) {
            unchecked {
                i--;
            }
            if (owners[i] == target) {
                found = true;
                idx = i;
                break;
            }
        }

        if (found) {
            assertEq(got.length, 1);
            assertEq(got[0], bytes32(idx + 1));
        } else {
            assertEq(got.length, 0);
        }
    }
}
