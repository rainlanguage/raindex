// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibOrderBookDeploy} from "src/lib/deploy/LibOrderBookDeploy.sol";

/// @title LibOrderBookDeploySubgraphYamlAddressTest
/// @notice The placeholder address in subgraph/subgraph.yaml MUST match the
/// deterministic OrderBook deploy address from LibOrderBookDeploy.
contract LibOrderBookDeploySubgraphYamlAddressTest is Test {
    function testSubgraphYamlAddress() external {
        string[] memory inputs = new string[](3);
        inputs[0] = "yq";
        inputs[1] = ".dataSources[0].source.address";
        inputs[2] = "subgraph/subgraph.yaml";
        bytes memory result = vm.ffi(inputs);
        address addr = address(bytes20(result));
        assertEq(addr, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS, "subgraph.yaml address mismatch");
    }
}
