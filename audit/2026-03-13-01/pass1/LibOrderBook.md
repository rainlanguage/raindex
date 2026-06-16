# Pass 1: Security -- LibOrderBook.sol

**Agent:** A11
**File:** src/lib/LibOrderBook.sol

## Evidence of Thorough Reading

**Library name:** `LibOrderBook` (line 96)

**Functions:**
| Line | Function |
|------|----------|
| 97 | `doPost(bytes32[][] memory context, TaskV2[] memory post) internal` |

This is the only function in the library.

**Types/Errors/Constants defined (file-level, outside the library):**

| Lines | Item | Kind |
|-------|------|------|
| 28 | `CALLING_CONTEXT_COLUMNS = 4` | constant |
| 30 | `CONTEXT_COLUMNS = CALLING_CONTEXT_COLUMNS + 1` | constant |
| 36 | `CONTEXT_CALLING_CONTEXT_COLUMN = 1` | constant |
| 37 | `CONTEXT_CALLING_CONTEXT_ROWS = 3` | constant |
| 39 | `CONTEXT_CALLING_CONTEXT_ROW_ORDER_HASH = 0` | constant |
| 40 | `CONTEXT_CALLING_CONTEXT_ROW_ORDER_OWNER = 1` | constant |
| 41 | `CONTEXT_CALLING_CONTEXT_ROW_ORDER_COUNTERPARTY = 2` | constant |
| 43 | `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_TOKEN = 0` | constant |
| 44 | `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_ID = 1` | constant |
| 45 | `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_BEFORE = 2` | constant |
| 46 | `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_AFTER = 3` | constant |
| 48 | `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_TOKEN = 0` | constant |
| 49 | `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_ID = 1` | constant |
| 50 | `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_BEFORE = 2` | constant |
| 51 | `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_AFTER = 3` | constant |
| 52 | `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_TARGET_AMOUNT = 4` | constant |
| 56 | `CONTEXT_CALCULATIONS_COLUMN = 2` | constant |
| 57 | `CONTEXT_CALCULATIONS_ROWS = 2` | constant |
| 59 | `CONTEXT_CALCULATIONS_ROW_MAX_OUTPUT = 0` | constant |
| 60 | `CONTEXT_CALCULATIONS_ROW_IO_RATIO = 1` | constant |
| 66 | `CONTEXT_VAULT_INPUTS_COLUMN = 3` | constant |
| 69 | `CONTEXT_VAULT_OUTPUTS_COLUMN = 4` | constant |
| 72 | `CONTEXT_VAULT_IO_TOKEN = 0` | constant |
| 74 | `CONTEXT_VAULT_IO_TOKEN_DECIMALS = 1` | constant |
| 76 | `CONTEXT_VAULT_IO_VAULT_ID = 2` | constant |
| 79 | `CONTEXT_VAULT_IO_BALANCE_BEFORE = 3` | constant |
| 84 | `CONTEXT_VAULT_IO_BALANCE_DIFF = 4` | constant |
| 86 | `CONTEXT_VAULT_IO_ROWS = 5` | constant |
| 88 | `CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN = 5` | constant |
| 89 | `CONTEXT_SIGNED_CONTEXT_SIGNERS_ROWS = 1` | constant |
| 90 | `CONTEXT_SIGNED_CONTEXT_SIGNERS_ROW = 0` | constant |
| 92 | `CONTEXT_SIGNED_CONTEXT_START_COLUMN = 6` | constant |
| 93 | `CONTEXT_SIGNED_CONTEXT_START_ROWS = 1` | constant |
| 94 | `CONTEXT_SIGNED_CONTEXT_START_ROW = 0` | constant |

No custom errors or structs are defined in this file. No assembly is used.

**Imports:**
- `CONTEXT_BASE_ROWS`, `CONTEXT_BASE_ROW_SENDER`, `CONTEXT_BASE_ROW_CALLING_CONTRACT`, `CONTEXT_BASE_COLUMN` from `LibContext.sol`
- `TaskV2` from `IRaindexV6.sol`
- `SourceIndexV2`, `StateNamespace`, `StackItem`, `EvalV4` from `IInterpreterV4.sol`
- `LibNamespace`, `FullyQualifiedNamespace` from `LibNamespace.sol`
- `LibContext` from `LibContext.sol`

## Findings

No CRITICAL, HIGH, MEDIUM, or LOW findings were identified. The analysis below documents what was checked and why no issues were found.

###Checklist of areas reviewed with no issues found:

**Reentrancy:** `doPost` makes external calls to `task.evaluable.interpreter.eval4(...)` and `task.evaluable.store.set(...)`. Both the interpreter and store addresses come from the user-supplied `TaskV2`. However, all callers of `doPost` in `OrderBookV6.sol` (`entask2`, `deposit4`, `withdraw4`, `addOrder4`, `removeOrder3`) are protected by `nonReentrant`. The `doPost` call is always made after all state changes (vault balance updates, order state changes) are complete, so even though external calls occur, all contract state is already finalized. This is correct.

**Namespace consistency:** `doPost` creates `namespace = StateNamespace.wrap(uint256(uint160(msg.sender)))` and then `qualifiedNamespace = LibNamespace.qualifyNamespace(namespace, address(this))`. The `qualifiedNamespace` is passed to `eval4` so the interpreter reads from the correct namespace. The unqualified `namespace` is passed to `store.set`, which is correct because `IInterpreterStoreV3.set` takes an unqualified namespace and the store implementation (`RainterpreterStore`) qualifies it internally with `msg.sender` (which is the OrderBook, i.e. `address(this)`). These produce identical fully qualified namespaces, so state reads during eval and state writes via set are consistent.

**Input validation / empty arrays:** When `post.length == 0`, the loop body never executes, which is correct. When `task.evaluable.bytecode.length == 0`, the eval is skipped entirely, which is correct. When `writes.length == 0`, the `store.set` call is skipped, which is correct and gas-efficient.

**Arithmetic safety:** The loop counter `++i` uses checked arithmetic (no `unchecked` block), so overflow is impossible. No other arithmetic operations exist.

**Memory safety:** No assembly is used. All memory operations use standard Solidity patterns. `emptyStack` and `emptyStateOverlay` are allocated once outside the loop and reused, which is safe since they are zero-length immutable arrays.

**Error handling:** External calls to `eval4` and `store.set` are not wrapped in try/catch. If either reverts, the entire transaction reverts, which is the intended behavior per the IRaindexV6 specification ("If ANY of the expressions revert, the entire transaction MUST revert").

**Stack suppression:** Line 118 `(stack);` suppresses the unused variable warning for the stack returned by `eval4`. The stack values from post-tasks are intentionally discarded. This is a standard Solidity pattern and not a bug.

**User-supplied interpreter/store addresses:** The `task.evaluable.interpreter` and `task.evaluable.store` are user-supplied external contract addresses. A malicious user could supply a malicious interpreter or store. However, this is by design -- users provide their own tasks and bear the risk of their own evaluable configurations. The namespace is derived from `msg.sender`, so one user's tasks cannot affect another user's state. This is documented in the `EvaluableV4` struct's NatSpec: "Callers MUST NOT use an `EvaluableV4` with a zero or untrusted interpreter address."

### A11-1 [INFO] Unused return value from `eval4` is silently discarded

**Location:** Line 118

Line 118 reads `(stack);` which discards the stack output from `eval4`. This is intentional for post-tasks (the stack output is not meaningful), but there is no comment explaining why the return value is discarded. Adding a brief comment would improve readability for future auditors and developers.

---

No fix files are required as the only finding is INFO-level.
