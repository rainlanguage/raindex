// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {ROUTE_PROCESSOR_4_CREATION_CODE} from "../../../src/lib/deploy/LibRouteProcessor4CreationCode.sol";

/// @dev Known codehash of the RouteProcessor4 runtime bytecode, verified
/// against the Sushi deployment on Ethereum mainnet.
bytes32 constant KNOWN_ROUTE_PROCESSOR_4_CODEHASH =
    bytes32(0xeb3745a79c6ba48e8767b9c355b8e7b79f9d6edeca004e4bb91be4de515a7eeb);

/// @title LibRouteProcessor4CreationCodeTest
/// @notice Deploys RouteProcessor4 from the stored creation code and verifies
/// that the resulting runtime bytecode hash matches the known Sushi deployment.
contract LibRouteProcessor4CreationCodeTest is Test {
    /// Deploying the stored creation code MUST produce runtime bytecode whose
    /// hash matches the known Sushi RouteProcessor4 codehash.
    function testRouteProcessor4Codehash() external {
        bytes memory creationCode = ROUTE_PROCESSOR_4_CREATION_CODE;
        address deployed;
        assembly ("memory-safe") {
            deployed := create(0, add(creationCode, 0x20), mload(creationCode))
        }
        assertTrue(deployed != address(0), "RouteProcessor4 deployment failed");
        assertEq(deployed.codehash, KNOWN_ROUTE_PROCESSOR_4_CODEHASH);
    }
}
