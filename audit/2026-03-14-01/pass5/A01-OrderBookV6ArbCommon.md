# A01 - Pass 5: Correctness / Intent Verification
## File: `src/abstract/OrderBookV6ArbCommon.sol`

## Evidence: Source Inventory

**Contract**: `OrderBookV6ArbCommon` (abstract, lines 29-60)

| Item | Kind | Line |
|------|------|------|
| `OrderBookV6ArbConfig` | struct | 13-16 |
| `WrongTask` | error | 19 |
| `BEFORE_ARB_SOURCE_INDEX` | constant | 23 |
| `Construct` | event | 35 |
| `iTaskHash` | immutable | 39 |
| `constructor` | function | 42-49 |
| `onlyValidTask` | modifier | 54-59 |

## Analysis

### Constants and Magic Numbers

**BEFORE_ARB_SOURCE_INDEX (line 23)**
- Declared as `SourceIndexV2.wrap(0)`.
- NatSpec says: "Before arb" is evaluated before the arb is executed.
- **Problem**: This constant is declared but never referenced in any production code. It is imported by one test file (`GenericPoolOrderBookV6ArbOrderTaker.expression.t.sol`) but even there it is unused -- only `WrongTask` from that import line is actually used. The `doPost` function in `LibOrderBook` hardcodes `SourceIndexV2.wrap(0)` directly rather than using this constant. The constant's NatSpec describes "before arb" semantics, but the actual usage context is "post arb" task evaluation in `doPost`. The name and documentation are misleading: there is no "before arb" evaluation in the current codebase.

### NatSpec vs. Implementation

1. **Struct `OrderBookV6ArbConfig` (line 10-16)**: NatSpec says `task` is "The task to run as post for each arb" and `implementationData` is "The constructor data for the specific implementation." Both match the constructor behavior. Correct.

2. **Error `WrongTask` (line 18-19)**: NatSpec says "Thrown when the task does not match the expected hash." The modifier `onlyValidTask` (line 55) reverts with `WrongTask` when `iTaskHash != bytes32(0) && iTaskHash != keccak256(abi.encode(task))`. This matches -- the error is thrown exactly when a non-zero task hash does not match the provided task. Correct.

3. **Event `Construct` (line 33-35)**: NatSpec says "Emitted on construction with the full config" with sender and config params. The constructor (line 44) emits `Construct(msg.sender, config)`. Correct.

4. **`iTaskHash` (line 37-39)**: NatSpec says "Hash of the configured task, or `bytes32(0)` if no task was set." The constructor sets it to `keccak256(abi.encode(config.task))` when bytecode is non-empty, and leaves it as the default `0` otherwise. Correct.

5. **`onlyValidTask` (line 51-58)**: NatSpec says "Reverts with `WrongTask` if `iTaskHash` is nonzero and does not match the hash of the provided task. Passes through if no task was configured." The implementation at line 55 checks `iTaskHash != bytes32(0) && iTaskHash != keccak256(abi.encode(task))`, which matches exactly: when iTaskHash is zero (no task configured), the first condition is false and execution passes through; when non-zero and mismatched, it reverts. Correct.

### Tests vs. Claims

1. **`testITaskHashNonEmpty`**: Claims iTaskHash equals keccak256 of the task when constructed with non-empty bytecode. Creates a task with `hex"deadbeef"` bytecode, deploys, and asserts equality. Correctly exercises the claim.

2. **`testITaskHashEmpty`**: Claims iTaskHash is bytes32(0) when constructed with empty bytecode. Creates a task with `hex""` bytecode, deploys, and asserts zero. Correctly exercises the claim.

3. **`testFallbackAcceptsCalldata` / `testFallbackAcceptsEmptyCalldata` / `testReceiveAcceptsETH` / `testFallbackAcceptsETHWithCalldata`** (in `OrderBookV6ArbCommon.fallback.t.sol`): These test the fallback behavior of the concrete `GenericPoolOrderBookV6ArbOrderTaker`, not `OrderBookV6ArbCommon` itself. The file is named `OrderBookV6ArbCommon.fallback.t.sol` but also contains `OrderBookV6FlashBorrowerFallbackTest`. The tests correctly exercise fallback/receive behavior on both contract types. Tests match their names.

### Interface Conformance

`OrderBookV6ArbCommon` does not claim to implement any interface. Correct.

### Error Conditions vs. Triggers

**WrongTask**: Triggered when `iTaskHash != bytes32(0) && iTaskHash != keccak256(abi.encode(task))`. The name "WrongTask" accurately describes this: the provided task is wrong (doesn't match). Correct.

## Findings

### A01-1: Unused constant `BEFORE_ARB_SOURCE_INDEX` with misleading NatSpec

**Severity**: LOW

`BEFORE_ARB_SOURCE_INDEX` at line 23 is declared as `SourceIndexV2.wrap(0)` with NatSpec describing "before arb" evaluation semantics. However:

1. The constant is never used in any production code. The only reference is an unused import in a test file.
2. The name and NatSpec describe "before arb" evaluation, but no such evaluation exists in the current codebase. The task evaluation in `LibOrderBook.doPost` happens *after* the arb, and it hardcodes `SourceIndexV2.wrap(0)` directly rather than referencing this constant.
3. This is dead code with misleading documentation that could confuse future developers or auditors into believing a "before arb" access control mechanism exists.

**Location**: `src/abstract/OrderBookV6ArbCommon.sol:21-23`
