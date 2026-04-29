# A11 - LibOrderBook.sol - Pass 4 (Code Quality)

**File:** `src/lib/LibOrderBook.sol`
**Lines:** 139

## Evidence Inventory

### Library
- `LibOrderBook` (library) - line 108

### Functions
- `doPost(bytes32[][] memory context, TaskV2[] memory post)` (internal) - line 111

### Imports (lines 5-19)
- `CONTEXT_BASE_ROWS`, `CONTEXT_BASE_ROW_SENDER`, `CONTEXT_BASE_ROW_CALLING_CONTRACT`, `CONTEXT_BASE_COLUMN` from `rain.interpreter.interface/lib/caller/LibContext.sol` - lines 6-10
- `TaskV2` from `rain.raindex.interface/interface/IRaindexV6.sol` - line 11
- `SourceIndexV2`, `StateNamespace`, `StackItem`, `EvalV4` from `rain.interpreter.interface/interface/IInterpreterV4.sol` - lines 13-17
- `LibNamespace`, `FullyQualifiedNamespace` from `rain.interpreter.interface/lib/ns/LibNamespace.sol` - line 18
- `LibContext` from `rain.interpreter.interface/lib/caller/LibContext.sol` - line 19

### Constants (file-level, lines 28-106)
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
- `CONTEXT_VAULT_IO_BALANCE_BEFORE` = 3 (line 86)
- `CONTEXT_VAULT_IO_BALANCE_DIFF` = 4 (line 92)
- `CONTEXT_VAULT_IO_ROWS` = 5 (line 94)
- `CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN` = 5 (line 98)
- `CONTEXT_SIGNED_CONTEXT_SIGNERS_ROWS` = 1 (line 99)
- `CONTEXT_SIGNED_CONTEXT_SIGNERS_ROW` = 0 (line 100)
- `CONTEXT_SIGNED_CONTEXT_START_COLUMN` = 6 (line 104)
- `CONTEXT_SIGNED_CONTEXT_START_ROWS` = 1 (line 105)
- `CONTEXT_SIGNED_CONTEXT_START_ROW` = 0 (line 106)

## Findings

### A11-1: Unused Imports from LibContext.sol [LOW]

Lines 6-10 import four symbols from `rain.interpreter.interface/lib/caller/LibContext.sol`:
- `CONTEXT_BASE_ROWS`
- `CONTEXT_BASE_ROW_SENDER`
- `CONTEXT_BASE_ROW_CALLING_CONTRACT`
- `CONTEXT_BASE_COLUMN`

These four constants are **not used within `LibOrderBook.sol` itself**. They are only consumed by `LibOrderBookSubParser.sol`, which imports them transitively via `"./LibOrderBook.sol"` (LibOrderBookSubParser.sol line 9).

Re-exporting symbols through a file that does not use them is a leaky abstraction. It couples `LibOrderBookSubParser` to `LibOrderBook` for symbols that `LibOrderBook` neither defines nor consumes. If these constants were removed from the import in LibOrderBook.sol, the downstream consumer would need to import them directly from their source, which is the correct dependency.

Similarly, `LibContext` is imported on line 19 but is only used in `doPost` (line 128 via `LibContext.build`). This import is legitimately used, so it is fine.

**Recommendation:** Move the four `CONTEXT_BASE_*` imports out of `LibOrderBook.sol`. Update `LibOrderBookSubParser.sol` to import them directly from `rain.interpreter.interface/lib/caller/LibContext.sol`.

---

**No other findings.** No commented-out code, no bare `src/` imports (all imports use remapped or relative paths), no style inconsistencies within the file. The `doPost` function is clean and well-structured. The pragma `^0.8.19` is consistent with sibling files.
