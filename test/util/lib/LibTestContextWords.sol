// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.25;

/// @dev Raindex sub parser context words as rainlang call expressions.
library LibTestContextWords {
    function depositWords() internal pure returns (string[] memory) {
        string[] memory words = new string[](6);
        words[0] = "depositor()";
        words[1] = "deposit-token()";
        words[2] = "deposit-vault-id()";
        words[3] = "deposit-vault-before()";
        words[4] = "deposit-vault-after()";
        words[5] = "deposit-token-decimals()";
        return words;
    }
}
