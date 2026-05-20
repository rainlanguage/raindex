// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6SubParserContextTest} from "test/util/abstract/RaindexV6SubParserContextTest.sol";

contract RaindexV6SubParserContextOrderOwnerTest is RaindexV6SubParserContextTest {
    function word() internal pure override returns (string memory) {
        return "order-owner";
    }
}
