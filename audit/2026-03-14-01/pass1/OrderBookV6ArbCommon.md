# A01: OrderBookV6ArbCommon.sol - Pass 1 (Security)

**File:** `src/abstract/OrderBookV6ArbCommon.sol`

## Evidence of Thorough Reading

**Contract:** `OrderBookV6ArbCommon` (abstract, line 29)

**Struct:**
- `OrderBookV6ArbConfig` (line 13): fields `task` (TaskV2), `implementationData` (bytes)

**Error:**
- `WrongTask()` (line 19)

**Constant:**
- `BEFORE_ARB_SOURCE_INDEX` (line 23): `SourceIndexV2.wrap(0)`

**Event:**
- `Construct(address sender, OrderBookV6ArbConfig config)` (line 35)

**State:**
- `iTaskHash` (line 39): `bytes32 public immutable`, initialized to 0

**Functions:**
- `constructor(OrderBookV6ArbConfig memory config)` (line 42)
- `onlyValidTask(TaskV2 memory task)` modifier (line 54)

**Imports:**
- `EvaluableV4`, `SignedContextV1` from `IInterpreterCallerV4.sol` (line 5)
- `SourceIndexV2` from `IInterpreterV4.sol` (line 6)
- `IRaindexV6`, `TaskV2` from `IRaindexV6.sol` (line 7)
- `LibEvaluable` from `LibEvaluable.sol` (line 8)

**Using:**
- `LibEvaluable for EvaluableV4` (line 30)

## Security Analysis

### Constructor (line 42-49)
- Emits `Construct` before any state changes or external calls -- correct ordering.
- Sets `iTaskHash` only if `config.task.evaluable.bytecode.length != 0`. Zero-length bytecode means no task gating.
- `keccak256(abi.encode(config.task))` is collision-resistant for structured data.

### onlyValidTask modifier (line 54-58)
- Short-circuits when `iTaskHash == bytes32(0)` (no task configured), allowing unrestricted access. This is documented behavior.
- When a task IS configured, requires exact hash match. Uses `keccak256(abi.encode(task))` which is deterministic for the same `TaskV2` struct.

### BEFORE_ARB_SOURCE_INDEX (line 23)
- Declared but not used within this file. It is a public constant for inheriting contracts. Not a security concern.

## Findings

No security findings. The contract is minimal and correct:
- Immutable task hash prevents post-deployment mutation.
- The modifier correctly gates access when a task is configured.
- No external calls, no state mutations beyond immutable initialization.
- Custom error `WrongTask()` is used (no string reverts).
