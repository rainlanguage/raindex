# Pass 3: Documentation -- A11 LibOrderBook

**File:** `src/lib/LibOrderBook.sol`

## Evidence of Reading

- **Library:** `LibOrderBook` (lines 108-139)
- **Imports:** `CONTEXT_BASE_*` from LibContext, `TaskV2`, `SourceIndexV2`, `StateNamespace`, `StackItem`, `EvalV4`, `LibNamespace`, `FullyQualifiedNamespace`, `LibContext` (lines 5-19)

### Public/Internal Functions

| Function | Visibility | Line | Has NatSpec |
|----------|-----------|------|-------------|
| `doPost(bytes32[][] memory context, TaskV2[] memory post)` | `internal` | 111 | Partial |

### Constants (file-level)

| Constant | Line | Has Doc |
|----------|------|---------|
| `CALLING_CONTEXT_COLUMNS` | 28 | Yes (lines 21-27) |
| `CONTEXT_COLUMNS` | 30 | No |
| `CONTEXT_COLUMNS_EXTENDED` | 34 | Yes (lines 32-33) |
| `CONTEXT_CALLING_CONTEXT_COLUMN` | 40 | Yes (lines 36-39) |
| `CONTEXT_CALLING_CONTEXT_ROWS` | 41 | No |
| `CONTEXT_CALLING_CONTEXT_ROW_ORDER_HASH` | 43 | No |
| `CONTEXT_CALLING_CONTEXT_ROW_ORDER_OWNER` | 44 | No |
| `CONTEXT_CALLING_CONTEXT_ROW_ORDER_COUNTERPARTY` | 45 | No |
| `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_TOKEN` | 49 | Yes (lines 47-48) |
| `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_ID` | 50 | No |
| `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_BEFORE` | 51 | No |
| `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_AFTER` | 52 | No |
| `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_TOKEN` | 56 | Yes (lines 54-55) |
| `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_ID` | 57 | No |
| `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_BEFORE` | 58 | No |
| `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_AFTER` | 59 | No |
| `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_TARGET_AMOUNT` | 60 | No |
| `CONTEXT_CALCULATIONS_COLUMN` | 64 | Yes (lines 62-63) |
| `CONTEXT_CALCULATIONS_ROWS` | 65 | No |
| `CONTEXT_CALCULATIONS_ROW_MAX_OUTPUT` | 67 | No |
| `CONTEXT_CALCULATIONS_ROW_IO_RATIO` | 68 | No |
| `CONTEXT_VAULT_INPUTS_COLUMN` | 74 | Yes (lines 70-73) |
| `CONTEXT_VAULT_OUTPUTS_COLUMN` | 77 | Yes (lines 75-76) |
| `CONTEXT_VAULT_IO_TOKEN` | 80 | Yes (line 79) |
| `CONTEXT_VAULT_IO_TOKEN_DECIMALS` | 82 | Yes (line 81) |
| `CONTEXT_VAULT_IO_VAULT_ID` | 84 | Yes (line 83) |
| `CONTEXT_VAULT_IO_BALANCE_BEFORE` | 87 | Yes (lines 85-86) |
| `CONTEXT_VAULT_IO_BALANCE_DIFF` | 92 | Yes (lines 88-91) |
| `CONTEXT_VAULT_IO_ROWS` | 94 | Yes (line 93) |
| `CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN` | 98 | Yes (lines 96-97) |
| `CONTEXT_SIGNED_CONTEXT_SIGNERS_ROWS` | 99 | No |
| `CONTEXT_SIGNED_CONTEXT_SIGNERS_ROW` | 100 | No |
| `CONTEXT_SIGNED_CONTEXT_START_COLUMN` | 104 | Yes (lines 102-103) |
| `CONTEXT_SIGNED_CONTEXT_START_ROWS` | 105 | No |
| `CONTEXT_SIGNED_CONTEXT_START_ROW` | 106 | No |

### Types / Errors

None declared in this file.

## Findings

### A11-1: `doPost` missing `@param` tags (INFO)

**Location:** Line 109-111

The `doPost` function has a `@dev` tag that describes its purpose ("Evaluates each task in `post` against the provided `context`. Tasks with empty bytecode are skipped."), but is missing explicit `@param` tags for its two parameters:

- `context` -- the context matrix passed to each task's evaluation
- `post` -- the array of tasks to evaluate

The `@dev` comment does mention both parameter names in prose, which provides some documentation, but formal `@param` tags would improve tooling support and consistency with Solidity NatSpec conventions.

### A11-2: `CONTEXT_COLUMNS` missing doc comment (INFO)

**Location:** Line 30

The constant `CONTEXT_COLUMNS` (value `CALLING_CONTEXT_COLUMNS + 1`) has no `@dev` comment. It is the total number of calling context columns plus the base column, but this is not documented.

### A11-3: Several row/count constants undocumented (INFO)

**Location:** Lines 41, 43-45, 50-52, 57-60, 65, 67-68, 99-100, 105-106

Multiple constants that represent row indices or row counts within context column groups lack individual `@dev` comments. The pattern is that the first constant in each group (the column identifier or the first row constant) has a block `@dev` comment, but subsequent row indices and row count values in the same group do not.

Constants without individual documentation:
- `CONTEXT_CALLING_CONTEXT_ROWS` (line 41)
- `CONTEXT_CALLING_CONTEXT_ROW_ORDER_HASH` (line 43)
- `CONTEXT_CALLING_CONTEXT_ROW_ORDER_OWNER` (line 44)
- `CONTEXT_CALLING_CONTEXT_ROW_ORDER_COUNTERPARTY` (line 45)
- `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_ID` (line 50)
- `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_BEFORE` (line 51)
- `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_AFTER` (line 52)
- `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_ID` (line 57)
- `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_BEFORE` (line 58)
- `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_AFTER` (line 59)
- `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_TARGET_AMOUNT` (line 60)
- `CONTEXT_CALCULATIONS_ROWS` (line 65)
- `CONTEXT_CALCULATIONS_ROW_MAX_OUTPUT` (line 67)
- `CONTEXT_CALCULATIONS_ROW_IO_RATIO` (line 68)
- `CONTEXT_SIGNED_CONTEXT_SIGNERS_ROWS` (line 99)
- `CONTEXT_SIGNED_CONTEXT_SIGNERS_ROW` (line 100)
- `CONTEXT_SIGNED_CONTEXT_START_ROWS` (line 105)
- `CONTEXT_SIGNED_CONTEXT_START_ROW` (line 106)

These constants have self-descriptive names and belong to documented groups, so the risk of misunderstanding is low. However, adding brief `@dev` comments would improve completeness.
