# A11: LibOrderBook.sol - Pass 1 (Security)

## Evidence of Thorough Reading

**File:** `src/lib/LibOrderBook.sol` (139 lines)

### Library
- `LibOrderBook` (line 108)

### Functions
- `doPost(bytes32[][] memory context, TaskV2[] memory post)` - line 111

### Constants (file-level)
- `CALLING_CONTEXT_COLUMNS` = 4 (line 28)
- `CONTEXT_COLUMNS` = CALLING_CONTEXT_COLUMNS + 1 (line 30)
- `CONTEXT_COLUMNS_EXTENDED` = CONTEXT_COLUMNS + 2 + 1 + 1 (line 34)
- `CONTEXT_CALLING_CONTEXT_COLUMN` = 1 (line 40)
- `CONTEXT_CALLING_CONTEXT_ROWS` = 3 (line 41)
- `CONTEXT_CALLING_CONTEXT_ROW_ORDER_HASH` = 0 (line 43)
- `CONTEXT_CALLING_CONTEXT_ROW_ORDER_OWNER` = 1 (line 44)
- `CONTEXT_CALLING_CONTEXT_ROW_ORDER_COUNTERPARTY` = 2 (line 45)
- `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_TOKEN` = 0 (line 49)
- `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_ID` = 1 (line 50)
- `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_BEFORE` = 2 (line 51)
- `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_AFTER` = 3 (line 52)
- `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_TOKEN` = 0 (line 54)
- `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_ID` = 1 (line 55)
- `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_BEFORE` = 2 (line 56)
- `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_AFTER` = 3 (line 57)
- `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_TARGET_AMOUNT` = 4 (line 60)
- `CONTEXT_CALCULATIONS_COLUMN` = 2 (line 64)
- `CONTEXT_CALCULATIONS_ROWS` = 2 (line 65)
- `CONTEXT_CALCULATIONS_ROW_MAX_OUTPUT` = 0 (line 67)
- `CONTEXT_CALCULATIONS_ROW_IO_RATIO` = 1 (line 68)
- `CONTEXT_VAULT_INPUTS_COLUMN` = 3 (line 74)
- `CONTEXT_VAULT_OUTPUTS_COLUMN` = 4 (line 77)
- `CONTEXT_VAULT_IO_TOKEN` = 0 (line 80)
- `CONTEXT_VAULT_IO_TOKEN_DECIMALS` = 1 (line 82)
- `CONTEXT_VAULT_IO_VAULT_ID` = 2 (line 84)
- `CONTEXT_VAULT_IO_BALANCE_BEFORE` = 3 (line 87)
- `CONTEXT_VAULT_IO_BALANCE_DIFF` = 4 (line 92)
- `CONTEXT_VAULT_IO_ROWS` = 5 (line 94)
- `CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN` = 5 (line 98)
- `CONTEXT_SIGNED_CONTEXT_SIGNERS_ROWS` = 1 (line 99)
- `CONTEXT_SIGNED_CONTEXT_SIGNERS_ROW` = 0 (line 100)
- `CONTEXT_SIGNED_CONTEXT_START_COLUMN` = 6 (line 104)
- `CONTEXT_SIGNED_CONTEXT_START_ROWS` = 1 (line 105)
- `CONTEXT_SIGNED_CONTEXT_START_ROW` = 0 (line 106)

### Imports
- `CONTEXT_BASE_ROWS`, `CONTEXT_BASE_ROW_SENDER`, `CONTEXT_BASE_ROW_CALLING_CONTRACT`, `CONTEXT_BASE_COLUMN` from LibContext
- `TaskV2` from IRaindexV6
- `SourceIndexV2`, `StateNamespace`, `StackItem`, `EvalV4` from IInterpreterV4
- `LibNamespace`, `FullyQualifiedNamespace` from LibNamespace
- `LibContext` from LibContext

## Findings

No security findings. The code is well-structured:

- **Namespace handling**: `doPost` derives `namespace` from `msg.sender` and passes it unqualified to `store.set`, which is correct per `IInterpreterStoreV3` specification -- the store MUST fully qualify internally.
- **Eval uses qualified namespace**: `eval4` receives `qualifiedNamespace` for read operations, consistent with interpreter expectations.
- **Empty bytecode check**: Tasks with empty bytecode are correctly skipped (line 119).
- **Write optimization**: Writes are only committed when non-empty (line 133).
- **No reentrancy risk**: External calls go to interpreter (`eval4`) and store (`set`), which are trusted components provided by the task. The caller context is not modified after external calls.
- **No assembly**: No assembly blocks in this library.
- **No string reverts**: No revert statements in this file.
