# Pass 1 Audit: OrderBookV6ArbCommon.sol

**Agent:** A01
**File:** `/Users/thedavidmeister/Code/rain.orderbook/src/abstract/OrderBookV6ArbCommon.sol`
**Date:** 2026-03-13

## Evidence of Thorough Reading

### Contract/Module Name
- `OrderBookV6ArbCommon` (abstract contract, line 34)

### Struct Definitions
- `OrderBookV6ArbConfig` (lines 21-25): fields `orderBook` (address), `task` (TaskV2), `implementationData` (bytes)

### Error Definitions
- `WrongTask()` (line 28)

### Constant Definitions
- `BEFORE_ARB_SOURCE_INDEX` (line 32): `SourceIndexV2.wrap(0)` -- note: defined at file scope but not used within this contract

### State Variables
- `iTaskHash` (line 39): `bytes32 public immutable`, initialized to `0`

### Events
- `Construct(address sender, OrderBookV6ArbConfig config)` (line 37)

### Functions/Modifiers (with line numbers)
- `constructor(OrderBookV6ArbConfig memory config)` (line 41)
- `modifier onlyValidTask(TaskV2 memory task)` (line 50)

### Imports (lines 5-14)
- `EvaluableV4`, `SignedContextV1` from `IInterpreterCallerV4.sol`
- `IInterpreterV4`, `SourceIndexV2`, `DEFAULT_STATE_NAMESPACE` from `IInterpreterV4.sol`
- `IRaindexV6`, `TaskV2` from `IRaindexV6.sol`
- `LibContext` from `LibContext.sol`
- `LibNamespace` from `LibNamespace.sol`
- `LibEvaluable` from `LibEvaluable.sol`
- `using LibEvaluable for EvaluableV4` (line 35)

## Security Review

### Memory Safety
No assembly blocks present. No inline assembly. No raw memory manipulation.

### Reentrancy
No external calls made in this contract. The constructor only emits an event and computes a keccak256 hash. The modifier `onlyValidTask` only performs comparisons and does not make external calls. Reentrancy is not a direct concern within this contract (inheriting contracts `OrderBookV6FlashBorrower` uses `ReentrancyGuard`).

### Access Control
The `onlyValidTask` modifier provides task validation. It is correctly applied by inheriting contracts.

### Arithmetic Safety
Only `keccak256` hashing is performed; no arithmetic operations. No overflow/underflow concerns.

### Input Validation
Task validation in the modifier checks the hash if `iTaskHash` is nonzero, and passes through if `iTaskHash` is zero (meaning no task was configured at construction time). This is intentional behavior.

## Findings

### A01-1: Unused Imports (INFO)

**Severity:** INFO

**Location:** Lines 5-14

**Description:** Several imports are not used within this contract:
- `IInterpreterV4` (line 7)
- `DEFAULT_STATE_NAMESPACE` (line 9)
- `IRaindexV6` (line 11)
- `LibContext` (line 12)
- `LibNamespace` (line 13)
- `BEFORE_ARB_SOURCE_INDEX` constant (line 32) is defined but not used within this contract

These are re-exported for use by inheriting contracts (`OrderBookV6ArbOrderTaker`, `OrderBookV6FlashBorrower`). While this is a common Solidity pattern to centralize imports, it can obscure actual dependencies and make auditing harder.

**Impact:** No security impact. Code clarity concern only.

### A01-2: `config.orderBook` Address Not Validated or Stored (LOW)

**Severity:** LOW

**Location:** Lines 41-48 (constructor)

**Description:** The `OrderBookV6ArbConfig` struct contains an `orderBook` address field (line 22), but the `OrderBookV6ArbCommon` constructor does not validate this address (e.g., checking for `address(0)`) nor does it store it as a state variable. The address is only emitted in the `Construct` event.

Inheriting contracts `OrderBookV6FlashBorrower` (line 65) and `OrderBookV6ArbOrderTaker` (line 39) each store `config.orderBook` as `iOrderBook` in their own constructors. However, neither validates it against `address(0)`.

If `address(0)` is passed as the orderbook address, the contract will deploy successfully and the error will only be discovered when actual arb operations are attempted, wasting gas on deployment.

**Impact:** Low. A misconfigured deployment would fail at first use rather than at construction. Since deployment is typically done by sophisticated actors (arb bots), the practical risk is minimal, but defense-in-depth suggests validating at construction time.
