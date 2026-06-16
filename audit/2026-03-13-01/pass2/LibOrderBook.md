# Pass 2: Test Coverage -- LibOrderBook.sol
**Agent:** A11
**Date:** 2026-03-13

## Source File Summary

**File:** `src/lib/LibOrderBook.sol` (125 lines)
**Library:** `LibOrderBook` (line 96)

### Imports
- `CONTEXT_BASE_ROWS`, `CONTEXT_BASE_ROW_SENDER`, `CONTEXT_BASE_ROW_CALLING_CONTRACT`, `CONTEXT_BASE_COLUMN` from `LibContext.sol`
- `TaskV2` from `IRaindexV6.sol`
- `SourceIndexV2`, `StateNamespace`, `StackItem`, `EvalV4` from `IInterpreterV4.sol`
- `LibNamespace`, `FullyQualifiedNamespace` from `LibNamespace.sol`
- `LibContext` from `LibContext.sol`

### Functions
| Function | Line | Visibility | Mutability | Description |
|----------|------|-----------|------------|-------------|
| `doPost(bytes32[][] memory context, TaskV2[] memory post)` | 97-124 | `internal` | (state-changing) | Iterates over `post` tasks, evals each with non-empty bytecode, writes state if writes are non-empty |

This is the only function in the library.

### Types/Errors/Constants
No custom errors, structs, or events are defined in this library. The file defines 30 file-level constants (lines 28-94) describing context layout columns and rows for order evaluation (calling context, calculations, vault I/O, signed context).

### Code Paths in `doPost`
1. **Empty post array:** Loop never executes (line 103)
2. **Task with empty bytecode:** `bytecode.length == 0` check skips eval (line 105)
3. **Task with bytecode, no writes:** Eval runs, writes are empty, `store.set` is skipped (line 119)
4. **Task with bytecode and writes:** Eval runs, `store.set` is called (lines 119-121)
5. **Signed context integration:** `LibContext.build(context, task.signedContext)` merges signed context into the context array (line 113)

## Test Coverage Map

There is no direct unit test file for `LibOrderBook`. The `doPost` function is tested indirectly through the following callers:

### Call Sites in Production Code
| Call Site | File | Line | Context Passed |
|-----------|------|------|----------------|
| `entask2` | `OrderBookV6.sol` | 255 | `new bytes32[][](0)` (empty) |
| `deposit4` | `OrderBookV6.sol` | 278 | 1-column matrix: token, vaultId, balanceBefore, balanceAfter, decimals |
| `withdraw4` | `OrderBookV6.sol` | 328 | 1-column matrix: token, vaultId, balanceBefore, balanceAfter, targetAmount, decimals |
| `addOrder4` | `OrderBookV6.sol` | 371 | 1-column matrix: orderHash, owner |
| `removeOrder3` | `OrderBookV6.sol` | 397 | 1-column matrix: orderHash, owner |
| `finalizeArb` | `LibOrderBookArb.sol` | 75 | 1-column matrix: inputBalance, outputBalance, gasBalance |

### Test Files Exercising `doPost`
| Test File | Entry Point | Code Paths Covered |
|-----------|-------------|-------------------|
| `OrderBookV6.entask.t.sol` | `entask2` | Paths 1,2,3,4 |
| `OrderBookV6.deposit.entask.t.sol` | `deposit4` | Paths 1,2,3,4 + context validation |
| `OrderBookV6.withdraw.entask.t.sol` | `withdraw4` | Paths 1,2,3,4 + context validation |
| `OrderBookV6.addOrder.entask.t.sol` | `addOrder4` | Paths 1,2,3,4 + context validation + noop when already live |
| `OrderBookV6.removeOrder.entask.t.sol` | `removeOrder3` | Paths 1,2,3,4 + noop when dead |
| `GenericPoolOrderBookV6ArbOrderTaker.expression.t.sol` | `arb5` (via `finalizeArb`) | Path 3,4 (mocked interpreter) |

### Per-Code-Path Coverage Detail

| Code Path | Covered | Where |
|-----------|---------|-------|
| 1. Empty post array | YES | `testOrderBookEvalEmptyNoop` and equivalent in all entask test files |
| 2. Empty bytecode task | PARTIAL | Implicitly -- no test explicitly creates a `TaskV2` with zero-length bytecode through a real entask call |
| 3. Eval with no writes | YES | `testOrderBookEvalOneStateless` (evals `_:1;` which produces no writes) |
| 4. Eval with writes | YES | `testOrderBookEvalWriteStateSingle`, `testOrderBookEvalWriteStateSequential` and equivalents in all entask files |
| 5. Signed context | NO | All entask tests pass `new SignedContextV1[](0)`. No test passes a non-empty `signedContext` array through a `doPost` path |

## Coverage Gaps

### CG-1: No test with non-empty `signedContext` in any `doPost` path (LOW)

**ID:** A11-P2-1

All test files that exercise `doPost` (entask, deposit, withdraw, addOrder, removeOrder) construct `TaskV2` with `new SignedContextV1[](0)`. The `doPost` function passes `task.signedContext` to `LibContext.build(context, task.signedContext)` (line 113), which merges signed context columns into the evaluation context. This code path is never tested through `doPost`.

While `LibContext.build` itself may be tested upstream in `rain.interpreter.interface`, and the `SignedContextV1` mechanism is tested through `takeOrders`/`clear` paths which use it during order evaluation (not through `doPost`), the specific integration of signed context with post-task evaluation is untested.

**Impact:** If `LibContext.build` had a bug specifically triggered when called from a post-task evaluation context (e.g., an interaction with the particular shape of the `context` array passed to `doPost`), it would not be caught by existing tests.

### CG-2: `testRemoveOrderContext` does not actually test context (LOW)

**ID:** A11-P2-2

In `test/concrete/ob/OrderBookV6.removeOrder.entask.t.sol`, the `testRemoveOrderContext` test (line 181) calls `checkRemoveOrder(alice, config, evals, 0, 0, false)` with `addOrder` = `false` (line 208). When `addOrder` is false, the order is never added so it is already dead. `removeOrder3` detects the order is dead (`sOrders[orderHash] != ORDER_LIVE`), returns `stateChanged = false`, and skips the call to `LibOrderBook.doPost` entirely. This means the context assertions in the eval strings (checking `orderbook()`, `order-hash()`, `order-owner()`) are **never actually executed**.

The test passes because `expectedReads = 0` and `expectedWrites = 0` -- which is correct for a noop where doPost is never called, but the test name and the eval strings suggest the intent was to validate context values during removeOrder post-evaluation.

**Impact:** The removeOrder context (orderHash, orderOwner) is not validated by any test. A bug in the context construction at `OrderBookV6.sol` lines 397-402 would not be caught.

### CG-3: No test for task revert atomicity through `entask2` (INFO)

**ID:** A11-P2-3

The `deposit.entask` and `withdraw.entask` test files both have explicit revert-in-action tests (`testDepositRevertInAction`, `testOrderBookWithdrawalEvalRevertInAction`) that verify a revert in a post-task reverts the entire transaction (including the deposit/withdraw). The `addOrder.entask` and `removeOrder.entask` files also have these tests. However, `OrderBookV6.entask.t.sol` (the standalone `entask2` path) has no revert test. Since `entask2` has no other state changes to roll back, a revert test would mainly confirm that the revert propagates out of `entask2`, which is trivially expected, but it would round out the test pattern.

### CG-4: No test for multiple tasks with different signed contexts (INFO)

**ID:** A11-P2-4

The `doPost` function iterates over an array of `TaskV2`, each potentially with its own `signedContext`. No test verifies that different tasks in the same `doPost` call can have different signed contexts that are independently evaluated.

### CG-5: Namespace isolation for `doPost` through arb path is not directly tested (INFO)

**ID:** A11-P2-5

When `doPost` is called from `LibOrderBookArb.finalizeArb`, the namespace is derived from `msg.sender` which is the arb bot address (not the order owner). The arb expression test (`GenericPoolOrderBookV6ArbOrderTaker.expression.t.sol`) mocks the interpreter entirely, so it does not test that state written by the arb's post-task is correctly namespaced under the arb bot's address. The `entask2` tests do validate namespace isolation between different `msg.sender` addresses, which covers the mechanism, but the arb-specific integration is untested with real interpreter.

## Findings Summary

| ID | Severity | Title |
|----|----------|-------|
| A11-P2-1 | LOW | No test with non-empty `signedContext` in any `doPost` path |
| A11-P2-2 | LOW | `testRemoveOrderContext` never executes its context assertions (dead order noop) |
| A11-P2-3 | INFO | No revert-propagation test for standalone `entask2` |
| A11-P2-4 | INFO | No test for multiple tasks with different signed contexts |
| A11-P2-5 | INFO | Arb-path namespace isolation not tested with real interpreter |
