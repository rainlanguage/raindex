# Pass 2 -- Test Coverage: A01 OrderBookV6ArbCommon

**Source file:** `src/abstract/OrderBookV6ArbCommon.sol` (60 lines)

## Source Summary

- `OrderBookV6ArbConfig` struct (task + implementationData)
- `WrongTask` error
- `BEFORE_ARB_SOURCE_INDEX` constant (SourceIndexV2.wrap(0))
- `iTaskHash` immutable bytes32 (defaults to 0)
- Constructor: emits `Construct`, sets `iTaskHash = keccak256(abi.encode(task))` when bytecode is non-empty
- `onlyValidTask` modifier: reverts `WrongTask` if iTaskHash != 0 and hash mismatch

## Test Files Found

| Test file | What it covers |
|-----------|---------------|
| `test/abstract/OrderBookV6ArbCommon.iTaskHash.t.sol` | iTaskHash non-empty and empty bytecode |
| `test/abstract/OrderBookV6ArbCommon.fallback.t.sol` | fallback/receive on OrderTaker and FlashBorrower |
| `test/abstract/OrderBookV6FlashBorrower.wrongTask.t.sol` | WrongTask revert on arb4 (FlashBorrower path) |
| `test/concrete/arb/GenericPoolOrderBookV6ArbOrderTaker.expression.t.sol` | WrongTask revert on arb5 (OrderTaker path) |
| `test/util/abstract/ArbTest.sol` | Construct event emission verified |

## Coverage Analysis

### Well-covered

- iTaskHash set correctly for non-empty bytecode (testITaskHashNonEmpty)
- iTaskHash is bytes32(0) for empty bytecode (testITaskHashEmpty)
- WrongTask revert on arb4 (FlashBorrower path) with fuzzed evaluable
- WrongTask revert on arb5 (OrderTaker path) with fuzzed evaluable
- Construct event emission (ArbTest constructor uses vm.expectEmit)
- Passthrough when iTaskHash is bytes32(0) (many tests pass empty bytecode)

### Gaps

**No gaps found.** All functions, error paths, and the modifier are exercised.
