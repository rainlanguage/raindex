# Pass 2: Test Coverage -- LibOrder.sol
**Agent:** A10
**Date:** 2026-03-13

## Source File Summary

**File:** `src/lib/LibOrder.sol` (19 lines)
**Library:** `LibOrder`

### Imports
- `OrderV4` from `rain.raindex.interface/interface/IRaindexV6.sol` (line 5)

### Functions
| Function | Line | Visibility | Mutability | Description |
|----------|------|-----------|------------|-------------|
| `hash(OrderV4 memory order)` | 16-18 | `internal` | `pure` | Returns `keccak256(abi.encode(order))` |

### Types/Errors/Constants/Events
None. The library defines no custom types, errors, constants, or events.

### OrderV4 Struct (external dependency)
```solidity
struct OrderV4 {
    address owner;
    EvaluableV4 evaluable;
    IOV2[] validInputs;
    IOV2[] validOutputs;
    bytes32 nonce;
}
```

## Test File Summary

**File:** `test/lib/LibOrder.t.sol` (23 lines)
**Contract:** `LibOrderTest is Test`

### Tests
| Test | Line | Type | Description |
|------|------|------|-------------|
| `testHashEqual(OrderV4 memory a)` | 14-16 | Fuzz (100 runs) | Asserts `hash(a) == hash(a)` |
| `testHashNotEqual(OrderV4 memory a, OrderV4 memory b)` | 20-22 | Fuzz (100 runs) | Asserts `hash(a) != hash(b)` |

## Coverage Gaps

### Function Coverage
- `hash`: Covered by both tests. **100% function coverage.**

### CG-1: `testHashNotEqual` is logically incorrect (LOW)

**ID:** A10-1

The test on lines 20-22 unconditionally asserts `LibOrder.hash(a) != LibOrder.hash(b)` for any fuzz-generated inputs `a` and `b`. If the fuzzer ever generates two identical `OrderV4` values, the assertion fails -- equal inputs must produce equal hashes. The test should include `vm.assume(keccak256(abi.encode(a)) != keccak256(abi.encode(b)))` or an equivalent guard.

In practice, with a complex struct containing dynamic arrays and 100 fuzz runs, collision probability is negligible, so the test likely never fails. However, the test's logical contract is wrong: it claims "different inputs produce different hashes" but does not actually ensure the inputs are different.

**Impact:** A fuzzer seed change, increased fuzz runs, or a targeted fuzzer could cause a spurious failure. More importantly, the test gives a false sense of coverage -- it does not actually prove collision resistance for distinct inputs because it never validates distinctness.

### CG-2: No known-value / reference hash test (INFO)

There is no test that computes a hash for a specific hardcoded `OrderV4` and compares against a known expected `bytes32` value. Such a test would serve as a regression guard: if the encoding method were accidentally changed (e.g., `abi.encode` to `abi.encodePacked`), it would catch the change.

### CG-3: No single-field-delta tests (INFO)

No tests verify that changing exactly one field of an `OrderV4` (e.g., only `nonce`, only `owner`) produces a different hash. While `abi.encode` is well-understood to be injective for distinct struct values, explicit single-field tests would provide targeted regression coverage and document the expected behavior for each field.

### CG-4: No empty-array edge case test (INFO)

No test explicitly covers orders with empty `validInputs` and/or `validOutputs` arrays. `abi.encode` handles empty dynamic arrays correctly, but an explicit test would document this assumption.

## Findings Summary

| ID | Severity | Title |
|----|----------|-------|
| A10-1 | LOW | `testHashNotEqual` lacks assumption guard -- asserts inequality without ensuring inputs differ |
| A10-2 | INFO | No known-value regression test for hash output |
| A10-3 | INFO | No single-field-delta collision tests |
| A10-4 | INFO | No empty-array edge case test |
