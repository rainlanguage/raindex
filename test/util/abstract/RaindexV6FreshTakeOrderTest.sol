// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6ExternalRealTest} from "test/util/abstract/RaindexV6ExternalRealTest.sol";
import {RaindexV6} from "src/concrete/raindex/RaindexV6.sol";
import {LibRaindexDeploy} from "../../../src/lib/deploy/LibRaindexDeploy.sol";

/// @title RaindexV6FreshTakeOrderTest
/// @notice Base for take-order tests that must observe mutations to
/// `src/concrete/raindex/RaindexV6.sol`. `RaindexV6ExternalRealTest` etches the
/// COMMITTED `src/generated/RaindexV6.pointers.sol` runtime bytecode (via
/// `LibEtchRaindex`) at `RAINDEX_DEPLOYED_ADDRESS`, which is blind to source
/// edits. This base re-etches that address with the FRESH-COMPILED
/// `type(RaindexV6).runtimeCode`, so a mutation to the take/skip loop is live in
/// what the tests execute. Orders are still added/parsed against the real etched
/// interpreter/store/parser, exactly as in the committed suite.
abstract contract RaindexV6FreshTakeOrderTest is RaindexV6ExternalRealTest {
    constructor() {
        // Re-etch the raindex address with freshly compiled runtime code so
        // source mutations to RaindexV6 are observed by the tests below.
        vm.etch(LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS, type(RaindexV6).runtimeCode);
    }
}
