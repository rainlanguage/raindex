// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.19;

import {Vm} from "forge-std/Vm.sol";
import {
    OrderV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV5,
    IOV2,
    EvaluableV4,
    OrderConfigV4,
    TaskV2,
    IRaindexV6
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rain.interpreter.interface/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/IInterpreterStoreV3.sol";
import {IParserV2} from "rain.interpreter.interface/interface/IParserV2.sol";
import {SignedContextV1} from "rain.interpreter.interface/interface/deprecated/v1/IInterpreterCallerV2.sol";
import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {LibInterpreterDeploy} from "rain.interpreter/lib/deploy/LibInterpreterDeploy.sol";
import {LibRaindexDeploy} from "../../../src/lib/deploy/LibRaindexDeploy.sol";

library LibTestTakeOrder {
    /// Extract OrderV4 from the first log entry emitted by addOrder4.
    function extractOrderFromLogs(Vm.Log[] memory entries) internal pure returns (OrderV4 memory) {
        (,, OrderV4 memory order) = abi.decode(entries[0].data, (address, bytes32, OrderV4));
        return order;
    }

    /// Parse an expression string, add the order as `owner`, and return the
    /// resulting OrderV4. Uses deploy constants for interpreter/store/parser
    /// and the raindex.
    function addOrderWithExpression(
        Vm vm,
        address owner,
        bytes memory expression,
        address inputToken,
        bytes32 inputVaultId,
        address outputToken,
        bytes32 outputVaultId
    ) internal returns (OrderV4 memory) {
        IParserV2 parser = IParserV2(LibInterpreterDeploy.EXPRESSION_DEPLOYER_DEPLOYED_ADDRESS);
        IRaindexV6 raindex = IRaindexV6(LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS);

        bytes memory bytecode = parser.parse2(expression);
        IOV2[] memory inputs = new IOV2[](1);
        inputs[0] = IOV2(inputToken, inputVaultId);
        IOV2[] memory outputs = new IOV2[](1);
        outputs[0] = IOV2(outputToken, outputVaultId);

        EvaluableV4 memory evaluable = EvaluableV4(
            IInterpreterV4(LibInterpreterDeploy.INTERPRETER_DEPLOYED_ADDRESS),
            IInterpreterStoreV3(LibInterpreterDeploy.STORE_DEPLOYED_ADDRESS),
            bytecode
        );
        OrderConfigV4 memory orderConfig = OrderConfigV4(evaluable, inputs, outputs, bytes32(0), bytes32(0), "");

        vm.prank(owner);
        vm.recordLogs();
        raindex.addOrder4(orderConfig, new TaskV2[](0));
        return extractOrderFromLogs(vm.getRecordedLogs());
    }

    /// Wrap a single order into a TakeOrderConfigV4 array with default IO
    /// indices (0, 0) and empty signed context.
    function wrapSingle(OrderV4 memory order) internal pure returns (TakeOrderConfigV4[] memory) {
        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](1);
        orders[0] = TakeOrderConfigV4(order, 0, 0, new SignedContextV1[](0));
        return orders;
    }

    /// Build a TakeOrdersConfigV5 with standard defaults: minimumIO=0,
    /// maximumIO=max, maximumIORatio=max, IOIsInput=true, data="".
    function defaultTakeConfig(TakeOrderConfigV4[] memory orders) internal pure returns (TakeOrdersConfigV5 memory) {
        return TakeOrdersConfigV5({
            minimumIO: LibDecimalFloat.packLossless(0, 0),
            maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
            maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
            IOIsInput: true,
            orders: orders,
            data: ""
        });
    }
}
