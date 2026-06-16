# Pass 5: Correctness -- OrderBookV6ArbCommon.sol

**Agent:** A01
**File:** `src/abstract/OrderBookV6ArbCommon.sol`

## Evidence of Thorough Reading

**Contract:** `OrderBookV6ArbCommon` (abstract, line 34)

### Types / Errors / Constants

| Kind | Name | Line |
|------|------|------|
| struct | `OrderBookV6ArbConfig` | 21-25 |
| error | `WrongTask()` | 28 |
| constant | `BEFORE_ARB_SOURCE_INDEX` | 32 |

### Functions / Modifiers

| Name | Kind | Line |
|------|------|------|
| `constructor(OrderBookV6ArbConfig)` | constructor | 41-48 |
| `onlyValidTask(TaskV2)` | modifier | 50-55 |

### Events

| Name | Line |
|------|------|
| `Construct(address, OrderBookV6ArbConfig)` | 37 |

### State Variables

| Name | Kind | Line |
|------|------|------|
| `iTaskHash` | `bytes32 public immutable` | 39 |

## Correctness Verification

### 1. `OrderBookV6ArbConfig` struct (lines 21-25)

**NatSpec claims:**
- `@param orderBook` -- "The `OrderBook` contract to arb against."
- `@param tasks` -- "The tasks to use as post for each arb." (Note: param name says `tasks` plural but actual field is `task` singular)
- `@param implementationData` -- "The constructor data for the specific implementation of the arb contract."

**Verification:** The struct contains `address orderBook`, `TaskV2 task`, and `bytes implementationData`. The `orderBook` field is not used within this contract (it is consumed by concrete implementations). The `task` field is hashed and stored. The `implementationData` field is consumed by concrete implementations.

**Finding:** The NatSpec says `@param tasks` (plural) but the field is `TaskV2 task` (singular). This is a doc mismatch.

### 2. `WrongTask` error (line 28)

**NatSpec claim:** "Thrown when the task does not match the expected hash."

**Verification:** Used in the `onlyValidTask` modifier at line 51. The modifier reverts with `WrongTask()` when `iTaskHash != bytes32(0) && iTaskHash != keccak256(abi.encode(task))`. This correctly throws when a non-zero task hash was set at construction and the provided task doesn't match.

**Tests:** `GenericPoolOrderBookV6ArbOrderTaker.expression.t.sol` (line 50) and `RouteProcessorOrderBookV6ArbOrderTaker.expression.t.sol` (line 42) both `vm.expectRevert(abi.encodeWithSelector(WrongTask.selector))` with mismatched evaluables. The test names (`testGenericPoolTakeOrdersWrongExpression`, `testRouteProcessorTakeOrdersWrongExpression`) describe the behavior they test. Verified: the tests provide an evaluable that differs from the one used at construction, and correctly expect `WrongTask`.

**Verdict:** Error name, documentation, and implementation all match. Tests confirm the behavior.

### 3. `BEFORE_ARB_SOURCE_INDEX` constant (line 32)

**NatSpec claim:** "'Before arb' is evaluated before the flash loan is taken. Ostensibly allows for some kind of access control to the arb."

**Verification:** The constant is set to `SourceIndexV2.wrap(0)`. It is not used within this contract. It is duplicated in `OrderBookV6ArbOrderTaker.sol` (line 29). Previously flagged across passes 1-4.

**Verdict:** The doc mentions "flash loan" specifically, but this contract is also inherited by `OrderBookV6ArbOrderTaker` which does not use flash loans. The doc is misleadingly narrow (flagged in pass 3 as A01-P3-3). The constant is unused in this file (flagged in pass 4 as A01-P4-2).

### 4. `iTaskHash` immutable (line 39)

**Behavior:** Initialized to `0` by default. In the constructor, if `config.task.evaluable.bytecode.length != 0`, it is set to `keccak256(abi.encode(config.task))`. Otherwise it stays `0`.

**Verification:** A zero `iTaskHash` means "no task validation" (the `onlyValidTask` modifier skips the check when `iTaskHash == bytes32(0)`). A non-zero `iTaskHash` enforces that the provided task matches what was set at construction. This is correct.

### 5. `onlyValidTask` modifier (lines 50-55)

**Logic:** `if (iTaskHash != bytes32(0) && iTaskHash != keccak256(abi.encode(task))) { revert WrongTask(); }`

**Verification:** When `iTaskHash == 0` (no task configured), ANY task is accepted, including one with non-empty bytecode. This is by design -- it allows permissionless arb bots to provide their own post-arb tasks. When `iTaskHash != 0`, only the exact task configured at construction is accepted.

**Correctness concern:** The modifier uses `abi.encode(task)` which encodes the entire `TaskV2` struct including `signedContext`. This means the signed context array must also match, not just the evaluable. For a task set at construction with an empty `signedContext`, callers must also provide an empty `signedContext`. This is correct and intentional -- it prevents callers from injecting arbitrary signed context into a fixed task.

### 6. Constructor (lines 41-48)

**Verification:** Emits `Construct` event before any external calls. Then conditionally sets `iTaskHash`. No external calls are made. The `Construct` event is tested in `ArbTest.sol` (line 82): `vm.expectEmit(); emit Construct(address(this), config);`.

**Verdict:** Correct.

### 7. `Construct` event (line 37)

**NatSpec:** None beyond the event signature.
**Verification:** Emitted in the constructor with `msg.sender` and the full config. Tested in `ArbTest.sol`. Correct.

## Findings

### A01-P5-1 [LOW] NatSpec `@param tasks` does not match field name `task`

**Severity:** LOW
**Confidence:** HIGH

**Location:** Line 18

The NatSpec reads `@param tasks` (plural) but the struct field is `TaskV2 task` (singular, line 22). This mismatch could confuse developers reading the documentation without looking at the code. It appears to be a leftover from when the struct may have held an array of tasks.

**Recommendation:** Change line 18 to `@param task The task to use as post for each arb.`

### A01-P5-2 [INFO] `onlyValidTask` accepts any task when no task hash is configured

When `iTaskHash == bytes32(0)` (no task bytecode at construction), the `onlyValidTask` modifier passes unconditionally, allowing callers to provide any arbitrary task including one with untrusted bytecode. This is by design for permissionless arb bots, but it is worth noting that an arb contract deployed without a task hash provides no post-arb validation. No fix needed; this is documenting intended behavior.

### A01-P5-3 [INFO] Constructor emits event before state mutation (good)

The constructor correctly emits `Construct` before setting `iTaskHash`, following the events-before-external-calls pattern. Since no external calls are made in this constructor, the ordering is informational only, but it is good practice.

## Summary Table

| ID | Severity | Title |
|----|----------|-------|
| A01-P5-1 | LOW | NatSpec `@param tasks` does not match field name `task` |
| A01-P5-2 | INFO | `onlyValidTask` accepts any task when no task hash configured |
| A01-P5-3 | INFO | Constructor event ordering is correct |
