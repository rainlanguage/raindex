// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6ExternalRealTest} from "test/util/abstract/RaindexV6ExternalRealTest.sol";
import {RaindexV6} from "../../../src/concrete/raindex/RaindexV6.sol";
import {LibRaindexDeploy} from "../../../src/lib/deploy/LibRaindexDeploy.sol";

/// @title RaindexV6FreshTest
/// @notice Same external setup as `RaindexV6ExternalRealTest` but overlays the
/// FRESHLY COMPILED `RaindexV6` runtime bytecode at the deployed address rather
/// than the committed `src/generated/RaindexV6.pointers.sol` runtime code that
/// `LibEtchRaindex` etches. The committed pointers code is frozen at the last
/// `BuildPointers.sol` run, so tests that run against it do not observe edits to
/// `src/concrete/raindex/RaindexV6.sol`. Overlaying `type(RaindexV6).runtimeCode`
/// makes the live source the code under test.
abstract contract RaindexV6FreshTest is RaindexV6ExternalRealTest {
    constructor() {
        // Overlay the freshly compiled runtime over the pointer-etched runtime.
        // `RaindexV6` has no constructor or immutables, so its runtime code is a
        // standalone, fully functional deployment.
        vm.etch(LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS, type(RaindexV6).runtimeCode);
    }
}
