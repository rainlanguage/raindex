# Pass 3: Documentation -- A13 LibOrderBookSubParser

**File:** `src/lib/LibOrderBookSubParser.sol`

## Evidence of Reading

**Library:** `LibOrderBookSubParser` (line 101)

**File-level constants (lines 49-96):**
- `SUB_PARSER_WORD_PARSERS_LENGTH` (line 49)
- `EXTERN_PARSE_META_BUILD_DEPTH` (line 50)
- `WORD_ORDER_CLEARER` (line 52)
- `WORD_ORDERBOOK` (line 53)
- `WORD_ORDER_HASH` (line 54)
- `WORD_ORDER_OWNER` (line 55)
- `WORD_ORDER_COUNTERPARTY` (line 56)
- `WORD_CALCULATED_MAX_OUTPUT` (line 57)
- `WORD_CALCULATED_IO_RATIO` (line 58)
- `WORD_INPUT_TOKEN` (line 59)
- `WORD_INPUT_TOKEN_DECIMALS` (line 60)
- `WORD_INPUT_VAULT_ID` (line 61)
- `WORD_INPUT_VAULT_BALANCE_BEFORE` (line 62)
- `WORD_INPUT_VAULT_BALANCE_INCREASE` (line 63)
- `WORD_OUTPUT_TOKEN` (line 64)
- `WORD_OUTPUT_TOKEN_DECIMALS` (line 65)
- `WORD_OUTPUT_VAULT_ID` (line 66)
- `WORD_OUTPUT_VAULT_BALANCE_BEFORE` (line 67)
- `WORD_OUTPUT_VAULT_BALANCE_DECREASE` (line 68)
- `WORD_DEPOSITOR` (line 70)
- `WORD_DEPOSIT_TOKEN` (line 71)
- `WORD_DEPOSIT_VAULT_ID` (line 72)
- `WORD_DEPOSIT_VAULT_BEFORE` (line 73)
- `WORD_DEPOSIT_VAULT_AFTER` (line 74)
- `WORD_WITHDRAWER` (line 76)
- `WORD_WITHDRAW_TOKEN` (line 77)
- `WORD_WITHDRAW_VAULT_ID` (line 78)
- `WORD_WITHDRAW_VAULT_BEFORE` (line 79)
- `WORD_WITHDRAW_VAULT_AFTER` (line 80)
- `WORD_WITHDRAW_TARGET_AMOUNT` (line 81)
- `DEPOSIT_WORD_DEPOSITOR` through `DEPOSIT_WORDS_LENGTH` (lines 83-88)
- `WITHDRAW_WORD_WITHDRAWER` through `WITHDRAW_WORDS_LENGTH` (lines 90-96)

**Functions (all `internal pure`):**
1. `subParserSender(uint256, uint256, OperandV2)` -- line 105, `@dev` on line 104
2. `subParserCallingContract(uint256, uint256, OperandV2)` -- line 111, `@dev` on line 110
3. `subParserOrderHash(uint256, uint256, OperandV2)` -- line 121, `@dev` on line 120
4. `subParserOrderOwner(uint256, uint256, OperandV2)` -- line 131, `@dev` on line 130
5. `subParserOrderCounterparty(uint256, uint256, OperandV2)` -- line 141, `@dev` on line 140
6. `subParserMaxOutput(uint256, uint256, OperandV2)` -- line 152, `@dev` on line 151
7. `subParserIORatio(uint256, uint256, OperandV2)` -- line 162, `@dev` on line 161
8. `subParserInputToken(uint256, uint256, OperandV2)` -- line 172, `@dev` on line 171
9. `subParserInputTokenDecimals(uint256, uint256, OperandV2)` -- line 182, `@dev` on line 181
10. `subParserInputVaultId(uint256, uint256, OperandV2)` -- line 192, `@dev` on line 191
11. `subParserInputBalanceBefore(uint256, uint256, OperandV2)` -- line 202, `@dev` on line 201
12. `subParserInputBalanceDiff(uint256, uint256, OperandV2)` -- line 212, `@dev` on line 211
13. `subParserOutputToken(uint256, uint256, OperandV2)` -- line 222, `@dev` on line 221
14. `subParserOutputTokenDecimals(uint256, uint256, OperandV2)` -- line 232, `@dev` on line 231
15. `subParserOutputVaultId(uint256, uint256, OperandV2)` -- line 242, `@dev` on line 241
16. `subParserOutputBalanceBefore(uint256, uint256, OperandV2)` -- line 252, `@dev` on line 251
17. `subParserOutputBalanceDiff(uint256, uint256, OperandV2)` -- line 262, `@dev` on line 261
18. `subParserSigners(uint256, uint256, OperandV2)` -- line 273, `@dev` on lines 271-272
19. `subParserDepositToken(uint256, uint256, OperandV2)` -- line 283, `@dev` on line 282
20. `subParserDepositVaultId(uint256, uint256, OperandV2)` -- line 293, `@dev` on line 292
21. `subParserDepositVaultBalanceBefore(uint256, uint256, OperandV2)` -- line 304, `@dev` on line 303
22. `subParserDepositVaultBalanceAfter(uint256, uint256, OperandV2)` -- line 317, `@dev` on line 316
23. `subParserWithdrawToken(uint256, uint256, OperandV2)` -- line 330, `@dev` on line 329
24. `subParserWithdrawVaultId(uint256, uint256, OperandV2)` -- line 340, `@dev` on line 339
25. `subParserWithdrawVaultBalanceBefore(uint256, uint256, OperandV2)` -- line 352, `@dev` on line 350
26. `subParserWithdrawVaultBalanceAfter(uint256, uint256, OperandV2)` -- line 364, `@dev` on line 363
27. `subParserWithdrawTargetAmount(uint256, uint256, OperandV2)` -- line 377, `@dev` on line 376
28. `subParserSignedContext(uint256, uint256, OperandV2)` -- line 390, `@dev` on lines 388-389
29. `authoringMetaV2()` -- line 406, `@dev` on lines 401-404

## Findings

### A13-1: No `@param` or `@return` documentation on any function (INFO)

**Severity:** INFO

All 29 functions have `@dev` comments describing their purpose but none document their parameters or return values. The functions share a common signature `(uint256, uint256, OperandV2) returns (bool, bytes memory, bytes32[] memory)` dictated by the sub-parser interface, and the first two `uint256` parameters are unnamed/unused throughout. While the interface-driven nature makes the parameters somewhat self-documenting, the return tuple `(bool, bytes memory, bytes32[] memory)` is not described anywhere in this file.

This is INFO because the `@dev` tags provide adequate context for what each function does, the signature is interface-driven, and the unnamed parameters indicate intentional disregard of those values.

### A13-2: File-level constants lack NatSpec documentation (INFO)

**Severity:** INFO

34 file-level constants (lines 49-96) have no NatSpec comments. These include:
- `SUB_PARSER_WORD_PARSERS_LENGTH` (line 49)
- `EXTERN_PARSE_META_BUILD_DEPTH` (line 50)
- All `WORD_*` constants (lines 52-81) -- these are self-documenting string literals
- All `DEPOSIT_WORD_*` and `WITHDRAW_WORD_*` index constants (lines 83-96)

The `WORD_*` constants are self-documenting because they are `bytes constant` whose values are the literal word strings they represent. The index constants (`DEPOSIT_WORD_*`, `WITHDRAW_WORD_*`) are straightforward enumerations.

`SUB_PARSER_WORD_PARSERS_LENGTH` and `EXTERN_PARSE_META_BUILD_DEPTH` are less obvious in purpose and would benefit from a brief doc comment, but are clearly integration constants consumed by the sub-parser framework.

This is INFO because the naming is highly descriptive and the values are self-evident.

### A13-3: `authoringMetaV2()` return value not documented (LOW)

**Severity:** LOW

The `authoringMetaV2()` function (line 406) has a `@dev` comment that says it "Returns ABI-encoded `AuthoringMetaV2[][]`" but the actual return is `bytes memory` after a flatten-and-encode operation (lines 621-631). The doc says it returns `AuthoringMetaV2[][]` but the code actually flattens it to `AuthoringMetaV2[]` before ABI encoding. The `@dev` comment is inaccurate: it says "ABI-encoded `AuthoringMetaV2[][]`" but the code flattens the 2D array into a 1D `AuthoringMetaV2[]` (line 625-628) and then encodes that flat array (line 631: `abi.encode(metaFlattened)`).

**Location:** Lines 401-404 vs lines 621-631.

The comment says:
> Returns ABI-encoded `AuthoringMetaV2[][]` covering every context column

The code does:
```
uint256[] memory metaUint256Flattened = metaUint256.flatten();
AuthoringMetaV2[] memory metaFlattened;
...
return abi.encode(metaFlattened);
```

The return is `abi.encode(AuthoringMetaV2[])`, not `abi.encode(AuthoringMetaV2[][])`.
