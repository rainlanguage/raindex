# Pass 3: Documentation -- LibOrderBook.sol

**Agent:** A11
**File:** `src/lib/LibOrderBook.sol` (126 lines)

## Evidence of Thorough Reading

- **Library name:** `LibOrderBook` (line 96)
- **Imports:** `CONTEXT_BASE_ROWS`, `CONTEXT_BASE_ROW_SENDER`, `CONTEXT_BASE_ROW_CALLING_CONTRACT`, `CONTEXT_BASE_COLUMN` from `LibContext.sol`; `TaskV2` from `IRaindexV6.sol`; `SourceIndexV2`, `StateNamespace`, `StackItem`, `EvalV4` from `IInterpreterV4.sol`; `LibNamespace`, `FullyQualifiedNamespace` from `LibNamespace.sol`; `LibContext` from `LibContext.sol` (lines 5-19)
- **Constants (file-level):**
  - `CALLING_CONTEXT_COLUMNS = 4` (line 28)
  - `CONTEXT_COLUMNS = CALLING_CONTEXT_COLUMNS + 1` (line 30)
  - `CONTEXT_CALLING_CONTEXT_COLUMN = 1` (line 36)
  - `CONTEXT_CALLING_CONTEXT_ROWS = 3` (line 37)
  - `CONTEXT_CALLING_CONTEXT_ROW_ORDER_HASH = 0` (line 39)
  - `CONTEXT_CALLING_CONTEXT_ROW_ORDER_OWNER = 1` (line 40)
  - `CONTEXT_CALLING_CONTEXT_ROW_ORDER_COUNTERPARTY = 2` (line 41)
  - `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_TOKEN = 0` (line 43)
  - `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_ID = 1` (line 44)
  - `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_BEFORE = 2` (line 45)
  - `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_AFTER = 3` (line 46)
  - `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_TOKEN = 0` (line 48)
  - `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_ID = 1` (line 49)
  - `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_BEFORE = 2` (line 50)
  - `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_AFTER = 3` (line 51)
  - `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_TARGET_AMOUNT = 4` (line 52)
  - `CONTEXT_CALCULATIONS_COLUMN = 2` (line 56)
  - `CONTEXT_CALCULATIONS_ROWS = 2` (line 57)
  - `CONTEXT_CALCULATIONS_ROW_MAX_OUTPUT = 0` (line 59)
  - `CONTEXT_CALCULATIONS_ROW_IO_RATIO = 1` (line 60)
  - `CONTEXT_VAULT_INPUTS_COLUMN = 3` (line 66)
  - `CONTEXT_VAULT_OUTPUTS_COLUMN = 4` (line 69)
  - `CONTEXT_VAULT_IO_TOKEN = 0` (line 72)
  - `CONTEXT_VAULT_IO_TOKEN_DECIMALS = 1` (line 74)
  - `CONTEXT_VAULT_IO_VAULT_ID = 2` (line 76)
  - `CONTEXT_VAULT_IO_BALANCE_BEFORE = 3` (line 79)
  - `CONTEXT_VAULT_IO_BALANCE_DIFF = 4` (line 84)
  - `CONTEXT_VAULT_IO_ROWS = 5` (line 86)
  - `CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN = 5` (line 88)
  - `CONTEXT_SIGNED_CONTEXT_SIGNERS_ROWS = 1` (line 89)
  - `CONTEXT_SIGNED_CONTEXT_SIGNERS_ROW = 0` (line 90)
  - `CONTEXT_SIGNED_CONTEXT_START_COLUMN = 6` (line 92)
  - `CONTEXT_SIGNED_CONTEXT_START_ROWS = 1` (line 93)
  - `CONTEXT_SIGNED_CONTEXT_START_ROW = 0` (line 94)
- **Functions:**
  - `doPost(bytes32[][] memory context, TaskV2[] memory post) internal` -- line 97

## Documentation Inventory

### Library-Level Documentation

| Item | Present? | Notes |
|---|---|---|
| `@title` | **No** | No `@title` tag on the library |
| `@notice` | **No** | No `@notice` tag on the library |

**Assessment:** The library itself has no NatSpec documentation. However, the file-level constants have extensive `@dev` documentation.

### Constants Documentation

Most constants have `@dev` NatSpec comments. The following is the assessment:

| Constant Group | Documented? | Notes |
|---|---|---|
| `CALLING_CONTEXT_COLUMNS` (line 28) | Yes | Via block comment lines 21-27 |
| `CONTEXT_COLUMNS` (line 30) | No | No doc comment |
| `CONTEXT_CALLING_CONTEXT_*` (lines 36-41) | Yes | Via block comment lines 32-35 |
| `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_*` (lines 43-46) | No | No doc comments |
| `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_*` (lines 48-52) | No | No doc comments |
| `CONTEXT_CALCULATIONS_*` (lines 56-60) | Yes | Via `@dev` on line 54-55 |
| `CONTEXT_VAULT_INPUTS_COLUMN` (line 66) | Yes | Via `@dev` on lines 62-65 |
| `CONTEXT_VAULT_OUTPUTS_COLUMN` (line 69) | Yes | Via `@dev` on lines 67-68 |
| `CONTEXT_VAULT_IO_*` (lines 72-86) | Yes | Each has `@dev` |
| `CONTEXT_SIGNED_CONTEXT_*` (lines 88-94) | No | No doc comments |

### Function: `doPost` (line 97)

| NatSpec Tag | Present? | Content |
|---|---|---|
| Description | **No** | No NatSpec at all |
| `@param context` | **No** | Missing |
| `@param post` | **No** | Missing |

**Assessment:** The sole function in this library has zero documentation. This is a significant gap given that `doPost` is the core post-evaluation dispatch function used by the OrderBook.

## Accuracy Check of Existing Documentation

1. **Line 23:** Typo "calculuate" should be "calculate".
2. **Line 82-83:** Typo "subtraced" should be "subtracted".
3. The `@dev` comment on `CALLING_CONTEXT_COLUMNS` (lines 21-27) explains the context column layout. It correctly notes that calling context is populated before calculate-order, with remaining columns only available to handle-IO. This is accurate.
4. The `@dev` comment on `CONTEXT_VAULT_IO_BALANCE_DIFF` (lines 80-83) says "The diff is ALWAYS POSITIVE as it is a `uint256` so it must be added to input balances and subtraced from output balances." This is accurate in intent (the diff is unsigned), though the typo should be fixed.

## Findings

### A11-P3-1: Missing NatSpec on `doPost` Function [LOW]

**Severity:** LOW

The `doPost` function at line 97 is the only function in the library and serves as the core post-evaluation dispatch mechanism. It has no NatSpec documentation at all -- no description, no `@param`, no `@return`. This function:
- Qualifies the namespace from `msg.sender`
- Iterates through post-tasks and evaluates each with non-empty bytecode
- Writes interpreter state if any writes are produced

Without documentation, developers must read the implementation to understand its behavior, the meaning of `context`, and the significance of namespace qualification from `msg.sender`.

### A11-P3-2: Typo "calculuate" in Constant Documentation [INFO]

**Severity:** INFO

Line 23: `/// available to handle IO as they depend on the full evaluation of calculuate` -- "calculuate" should be "calculate".

### A11-P3-3: Typo "subtraced" in Constant Documentation [INFO]

**Severity:** INFO

Line 82: `/// `uint256` so it must be added to input balances and subtraced from output` -- "subtraced" should be "subtracted".

### A11-P3-4: Missing Documentation on Deposit/Withdraw and Signed Context Constants [INFO]

**Severity:** INFO

The deposit-related context constants (lines 43-46), withdraw-related context constants (lines 48-52), and signed context constants (lines 88-94) have no `@dev` comments, unlike most other constants in the file. For consistency and maintainability, these should have brief `@dev` descriptions explaining their role.
