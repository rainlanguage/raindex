# A01 - Pass 4 (Code Quality) - OrderBookV6ArbCommon.sol

**File:** `src/abstract/OrderBookV6ArbCommon.sol`

## Evidence: Full Inventory

- **Contract:** `OrderBookV6ArbCommon` (abstract, line 29)
- **Struct:** `OrderBookV6ArbConfig` (line 13)
- **Error:** `WrongTask` (line 19)
- **Constant:** `BEFORE_ARB_SOURCE_INDEX` (line 23)
- **Event:** `Construct(address, OrderBookV6ArbConfig)` (line 35)
- **State variable:** `iTaskHash` (immutable, line 39)
- **Constructor:** lines 42-49
- **Modifier:** `onlyValidTask(TaskV2)` (line 54)
- **Using directive:** `using LibEvaluable for EvaluableV4` (line 30)
- **Imports:**
  - `EvaluableV4`, `SignedContextV1` from `rain.interpreter.interface/.../IInterpreterCallerV4.sol` (line 5)
  - `SourceIndexV2` from `rain.interpreter.interface/.../IInterpreterV4.sol` (line 6)
  - `IRaindexV6`, `TaskV2` from `rain.raindex.interface/.../IRaindexV6.sol` (line 7)
  - `LibEvaluable` from `rain.interpreter.interface/.../LibEvaluable.sol` (line 8)

## Findings

### A01-1: Unused `using LibEvaluable for EvaluableV4` directive (LOW)

**Line 30:** The `using LibEvaluable for EvaluableV4` directive is declared but no method from `LibEvaluable` is ever called on an `EvaluableV4` instance anywhere in this contract or its inheriting contracts (`OrderBookV6ArbOrderTaker`, `OrderBookV6FlashBorrower`, or their concrete implementations). This is dead code that adds unnecessary bytecode to deployments and obscures the actual interface surface.

The corresponding import of `LibEvaluable` (line 8) and the import of `EvaluableV4` (line 5) are only needed for this unused directive. `EvaluableV4` is accessed at line 46 via `config.task.evaluable.bytecode.length` which is direct struct field access and does not require the `using` directive.

### A01-2: Unused `BEFORE_ARB_SOURCE_INDEX` constant (LOW)

**Line 23:** The file-level constant `BEFORE_ARB_SOURCE_INDEX` is defined as `SourceIndexV2.wrap(0)` but is never referenced in any production source file under `src/`. It is only imported by one test file (`test/concrete/arb/GenericPoolOrderBookV6ArbOrderTaker.expression.t.sol`). The corresponding import of `SourceIndexV2` (line 6) exists solely for this constant.

A constant that only serves tests should live in test infrastructure, not in production code. Its presence here forces `SourceIndexV2` to be imported, increasing the conceptual surface area of the contract without providing production value.

### A01-3: Unused `SignedContextV1` import (LOW)

**Line 5:** `SignedContextV1` is imported alongside `EvaluableV4` but is never used in this file. It is re-exported from `OrderBookV6ArbOrderTaker.sol` (line 11) which also never uses it. This is a vestigial import from a previous version of the contract.

### A01-4: Unused `IRaindexV6` import (LOW)

**Line 7:** `IRaindexV6` is imported but never used directly in this file. It is not needed here -- the child contracts import it from their own direct dependency paths.
