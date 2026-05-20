// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {RaindexV6ExternalRealTest} from "test/util/abstract/RaindexV6ExternalRealTest.sol";
import {LibTestTakeOrder} from "test/util/lib/LibTestTakeOrder.sol";
import {OrderV4, TakeOrdersConfigV5, TaskV2} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {MinimumIO} from "../../../src/concrete/raindex/RaindexV6.sol";

/// When `IOIsInput = false`, `minimumIO` is checked against `totalTakerOutput`.
/// Verify the revert fires correctly in this branch.
contract RaindexV6TakeOrderMinimumIOIsOutputTest is RaindexV6ExternalRealTest {
    function testTakeOrderMinimumIOIsOutputRevert() external {
        address alice = address(uint160(uint256(keccak256("alice.rain.test"))));
        address bob = address(uint160(uint256(keccak256("bob.rain.test"))));

        // Deposit to alice's output vault.
        vm.mockCall(
            address(iToken1),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(iRaindex)),
            abi.encode(true)
        );
        vm.prank(alice);
        iRaindex.deposit4(
            address(iToken1), bytes32(uint256(0x01)), LibDecimalFloat.packLossless(1, 0), new TaskV2[](0)
        );

        // Order outputs 1e-18 at ratio 1.
        OrderV4 memory order = LibTestTakeOrder.addOrderWithExpression(
            vm,
            alice,
            "_ _:1e-18 1;:;",
            address(iToken0),
            bytes32(uint256(0x01)),
            address(iToken1),
            bytes32(uint256(0x01))
        );

        // IOIsInput = false means minimumIO is checked against totalTakerOutput.
        TakeOrdersConfigV5 memory takeConfig = LibTestTakeOrder.defaultTakeConfig(LibTestTakeOrder.wrapSingle(order));
        takeConfig.IOIsInput = false;
        takeConfig.minimumIO = LibDecimalFloat.packLossless(1, 0);

        vm.prank(bob);
        vm.expectRevert(
            abi.encodeWithSelector(
                MinimumIO.selector, LibDecimalFloat.packLossless(1, 0), LibDecimalFloat.packLossless(1, -18)
            )
        );
        iRaindex.takeOrders4(takeConfig);
    }
}
