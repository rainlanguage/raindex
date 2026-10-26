// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.25;

/// @dev Every Raindex sub parser context word as a rainlang call expression.
library LibTestContextWords {
    function allWords() internal pure returns (string[] memory) {
        string[] memory words = new string[](30);
        words[0] = "raindex()";
        words[1] = "order-hash()";
        words[2] = "order-owner()";
        words[3] = "order-counterparty()";
        words[4] = "order-clearer()";
        words[5] = "calculated-max-output()";
        words[6] = "calculated-io-ratio()";
        words[7] = "input-token()";
        words[8] = "input-token-decimals()";
        words[9] = "input-vault-id()";
        words[10] = "input-vault-before()";
        words[11] = "input-vault-increase()";
        words[12] = "output-token()";
        words[13] = "output-token-decimals()";
        words[14] = "output-vault-id()";
        words[15] = "output-vault-before()";
        words[16] = "output-vault-decrease()";
        words[17] = "depositor()";
        words[18] = "deposit-token()";
        words[19] = "deposit-vault-id()";
        words[20] = "deposit-vault-before()";
        words[21] = "deposit-vault-after()";
        words[22] = "deposit-token-decimals()";
        words[23] = "withdrawer()";
        words[24] = "withdraw-token()";
        words[25] = "withdraw-vault-id()";
        words[26] = "withdraw-vault-before()";
        words[27] = "withdraw-vault-after()";
        words[28] = "withdraw-target-amount()";
        words[29] = "withdraw-token-decimals()";
        return words;
    }
}
