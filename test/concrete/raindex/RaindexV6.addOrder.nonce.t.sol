// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6ExternalRealTest} from "test/util/abstract/RaindexV6ExternalRealTest.sol";
import {OrderConfigV4, OrderV4, EvaluableV4, TaskV2} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {LibOrder} from "../../../src/lib/LibOrder.sol";

contract RaindexV6AddOrderNonceTest is RaindexV6ExternalRealTest {
    using LibOrder for OrderV4;

    /// forge-config: default.fuzz.runs = 100
    function testAddOrderNonceSameOrderNoop(address owner, OrderConfigV4 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        OrderV4 memory order = OrderV4(owner, config.evaluable, config.validInputs, config.validOutputs, config.nonce);

        vm.prank(owner);
        bool stateChange = iRaindex.addOrder4(config, new TaskV2[](0));
        assert(stateChange);
        assert(iRaindex.orderExists(order.hash()));

        vm.prank(owner);
        stateChange = iRaindex.addOrder4(config, new TaskV2[](0));
        assert(!stateChange);
        assert(iRaindex.orderExists(order.hash()));
    }

    /// forge-config: default.fuzz.runs = 100
    function testAddOrderNonceDifferentNonceStateChange(address owner, OrderConfigV4 memory config, bytes32 otherNonce)
        public
    {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        vm.assume(config.nonce != otherNonce);
        vm.prank(owner);
        bool stateChange = iRaindex.addOrder4(config, new TaskV2[](0));
        assert(stateChange);
        assert(
            iRaindex.orderExists(
                OrderV4(owner, config.evaluable, config.validInputs, config.validOutputs, config.nonce).hash()
            )
        );

        config.nonce = otherNonce;
        vm.prank(owner);
        stateChange = iRaindex.addOrder4(config, new TaskV2[](0));
        assert(stateChange);
        assert(
            iRaindex.orderExists(
                OrderV4(owner, config.evaluable, config.validInputs, config.validOutputs, otherNonce).hash()
            )
        );
    }

    /// forge-config: default.fuzz.runs = 100
    function testAddOrderNonceSameNonceDifferentOrderStateChange(
        address owner,
        OrderConfigV4 memory config0,
        OrderConfigV4 memory config1
    ) public {
        LibTestAddOrder.conformConfig(config0, iInterpreter, iStore);
        LibTestAddOrder.conformConfig(config1, iInterpreter, iStore);

        config1.nonce = config0.nonce;
        vm.assume(keccak256(abi.encode(config0)) != keccak256(abi.encode(config1)));
        vm.prank(owner);
        bool stateChange = iRaindex.addOrder4(config0, new TaskV2[](0));
        assert(stateChange);
        assert(
            iRaindex.orderExists(
                OrderV4(owner, config0.evaluable, config0.validInputs, config0.validOutputs, config0.nonce).hash()
            )
        );

        vm.prank(owner);
        stateChange = iRaindex.addOrder4(config1, new TaskV2[](0));
        assert(stateChange);
        assert(
            iRaindex.orderExists(
                OrderV4(owner, config1.evaluable, config1.validInputs, config1.validOutputs, config1.nonce).hash()
            )
        );
    }
}
