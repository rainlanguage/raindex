# Pass 2: Test Coverage -- OrderBookV6ArbCommon.sol

**Agent:** A01
**File:** `src/abstract/OrderBookV6ArbCommon.sol`
**Date:** 2026-03-13

## Evidence of Thorough Reading

### Struct (file scope)
- `OrderBookV6ArbConfig` (lines 21-25): fields `orderBook` (address), `task` (TaskV2), `implementationData` (bytes)

### Error (file scope)
- `WrongTask()` (line 28)

### Constant (file scope)
- `BEFORE_ARB_SOURCE_INDEX` (line 32): `SourceIndexV2.wrap(0)`

### Contract: `OrderBookV6ArbCommon` (abstract, line 34)

| Item | Kind | Line |
|------|------|------|
| `using LibEvaluable for EvaluableV4` | using directive | 35 |
| `event Construct(address sender, OrderBookV6ArbConfig config)` | event | 37 |
| `iTaskHash` | `bytes32 public immutable` | 39 |
| `constructor(OrderBookV6ArbConfig memory config)` | constructor | 41 |
| `modifier onlyValidTask(TaskV2 memory task)` | modifier | 50 |

### Constructor Logic (lines 41-48)
1. Emits `Construct(msg.sender, config)` (line 43)
2. If `config.task.evaluable.bytecode.length != 0`, sets `iTaskHash = keccak256(abi.encode(config.task))` (lines 45-47)
3. Otherwise `iTaskHash` remains `bytes32(0)` (the initializer on line 39)

### Modifier Logic (lines 50-55)
1. If `iTaskHash != bytes32(0)` AND `iTaskHash != keccak256(abi.encode(task))`, reverts with `WrongTask()` (lines 51-53)
2. Otherwise execution continues

## Existing Test Files

| Test File | What It Tests |
|-----------|---------------|
| `test/util/abstract/ArbTest.sol` | Base test harness: constructs arb with non-empty expression `""`, verifies `Construct` event emission |
| `test/concrete/arb/GenericPoolOrderBookV6ArbOrderTaker.expression.t.sol` | `WrongTask` revert on mismatched evaluable; successful eval with matching task |
| `test/concrete/arb/RouteProcessorOrderBookV6ArbOrderTaker.expression.t.sol` | Same as above for RouteProcessor variant |
| `test/concrete/arb/GenericPoolOrderBookV6ArbOrderTaker.sender.t.sol` | Happy path with empty bytecode (no task hash) |
| `test/concrete/arb/RouteProcessorOrderBookV6ArbOrderTaker.sender.t.sol` | Same for RouteProcessor |
| `test/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sender.t.sol` | Happy path flash borrower with empty bytecode |
| `test/util/concrete/ChildOrderBookV6ArbOrderTaker.sol` | Deployable child with all-zero config |
| `test/abstract/OrderBookV6ArbOrderTaker.ierc165.t.sol` | ERC165 interface checks |
| `test/abstract/OrderBookV6FlashBorrower.ierc165.t.sol` | ERC165 interface checks |
| `test/abstract/OrderBookV6ArbOrderTaker.context.t.sol` | Context columns passed to task evaluable |

## Coverage Analysis

### What IS Tested

1. **`WrongTask` revert path (modifier `onlyValidTask`)**: Tested in both `GenericPoolOrderBookV6ArbOrderTaker.expression.t.sol` and `RouteProcessorOrderBookV6ArbOrderTaker.expression.t.sol`. These fuzz the evaluable to ensure any mismatched task triggers revert.

2. **Happy path with non-empty bytecode**: `testGenericPoolTakeOrdersExpression` and `testRouteProcessorTakeOrdersExpression` test that a matching task passes the modifier and the evaluable is called.

3. **Happy path with empty bytecode (iTaskHash == 0)**: The `*.sender.t.sol` tests and `GenericPoolOrderBookV6FlashBorrower.sender.t.sol` all construct with empty expression (via default `ArbTest` which uses `expression()` returning `""`), then call arb with an empty-bytecode task -- the modifier passes because `iTaskHash` is `bytes32(0)`.

4. **`Construct` event emission**: `ArbTest.sol` line 81-82 uses `vm.expectEmit()` to verify the event.

### What is NOT Tested

See findings below.

## Findings

### A01-P2-1 [LOW] No Test for `WrongTask` Revert Through FlashBorrower Path

**Location:** `src/abstract/OrderBookV6ArbCommon.sol:50-55` (modifier), tested only via `OrderBookV6ArbOrderTaker.arb5`

The `onlyValidTask` modifier is tested for `WrongTask` revert only through the `ArbOrderTaker.arb5` code path (in `GenericPoolOrderBookV6ArbOrderTaker.expression.t.sol` and `RouteProcessorOrderBookV6ArbOrderTaker.expression.t.sol`).

There is no test that verifies `WrongTask` is correctly triggered through the `OrderBookV6FlashBorrower.arb4` code path. While the modifier is inherited identically by both abstract contracts, changes to either subclass could break the modifier application without test detection.

The `GenericPoolOrderBookV6FlashBorrower.sender.t.sol` only tests the happy path (empty bytecode, `iTaskHash == 0`).

### A01-P2-2 [LOW] No Direct Unit Test for Constructor `iTaskHash` Assignment

**Location:** `src/abstract/OrderBookV6ArbCommon.sol:41-48` (constructor)

There is no test that directly asserts the value of `iTaskHash` after construction. The existing tests verify behavior (the modifier reverts or passes), but never read `iTaskHash` (which is a public immutable, so it has a getter) to confirm:
- That `iTaskHash == bytes32(0)` when constructed with empty bytecode
- That `iTaskHash == keccak256(abi.encode(task))` when constructed with non-empty bytecode

This is a defense-in-depth concern: if the constructor logic were accidentally changed (e.g., always setting the hash, or never setting it), the modifier tests might still pass depending on how the change manifests. A direct assertion on the stored value provides independent verification.

### A01-P2-3 [LOW] No Test for `onlyValidTask` Bypass When `iTaskHash == 0` With Arbitrary Non-Empty Task

**Location:** `src/abstract/OrderBookV6ArbCommon.sol:50-55` (modifier)

When the contract is constructed with empty bytecode (`iTaskHash == 0`), the modifier should allow ANY task through -- including tasks with non-empty, arbitrary bytecode. The existing `*.sender.t.sol` tests only pass an empty-bytecode task when `iTaskHash == 0`. There is no fuzz test confirming that arbitrary non-empty tasks also pass when `iTaskHash == 0`.

This matters because the modifier condition is `iTaskHash != bytes32(0) && ...` -- if someone refactored this to `||` or removed the first condition, it would not be caught by existing tests.

### A01-P2-4 [INFO] No Dedicated Test File for `OrderBookV6ArbCommon`

**Location:** N/A (test organization)

`OrderBookV6ArbCommon` has no dedicated test file. All testing of its behavior is done indirectly through tests of its concrete subclasses. While this provides functional coverage, it means:
- There is no single place to look for the contract's test coverage
- Adding new functionality to `OrderBookV6ArbCommon` requires remembering to test through multiple subclass paths
- The contract's invariants are not documented in test form

### A01-P2-5 [INFO] `BEFORE_ARB_SOURCE_INDEX` Constant Not Tested

**Location:** `src/abstract/OrderBookV6ArbCommon.sol:32`

The constant `BEFORE_ARB_SOURCE_INDEX = SourceIndexV2.wrap(0)` is defined at file scope but is not directly tested for its value. The constant is consumed by `OrderBookV6ArbOrderTaker` and `OrderBookV6FlashBorrower` in the `_beforeArb` logic (via `LibOrderBookArb`). While `OrderBookV6ArbOrderTaker.context.t.sol` exercises the before-arb path, there is no assertion that specifically validates the source index value.
