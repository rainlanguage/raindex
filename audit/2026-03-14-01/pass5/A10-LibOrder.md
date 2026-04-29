# A10 - LibOrder.sol - Pass 5 (Correctness / Intent Verification)

**Source file:** `src/lib/LibOrder.sol` (19 lines)
**Test file:** `test/lib/LibOrder.t.sol` (23 lines)

## Evidence Inventory

### LibOrder.sol
- **Library:** `LibOrder` (line 10)
- **Function:** `hash(OrderV4 memory order) -> bytes32` (line 16)
  - Pure, internal
  - Returns `keccak256(abi.encode(order))`

### Constants / Types / Errors
- None defined in this file.

### Imports
- `OrderV4` from `rain.raindex.interface/interface/IRaindexV6.sol`

---

## NatSpec vs. Implementation

### `LibOrder` library (line 7-9)
- **NatSpec claim:** "Consistent handling of `OrderV4` for where it matters w.r.t. determinism and security."
- **Verified:** The library provides exactly one function (`hash`) that deterministically hashes an `OrderV4`. Correct.

### `hash` function (line 11-17)
- **NatSpec claim:** "Hashes `OrderV4` in a secure and deterministic way. Uses abi.encode rather than abi.encodePacked to guard against potential collisions where many inputs encode to the same output bytes."
- **Verified:** Uses `keccak256(abi.encode(order))`. `abi.encode` pads each field to 32 bytes, preventing the collision issue that `abi.encodePacked` can cause with adjacent dynamic-length fields. Correct.
- **NatSpec claim:** "@param order The order to hash." / "@return The hash of `order`."
- **Verified:** Single input `OrderV4 memory order`, single output `bytes32`. Correct.

---

## Test Correctness

### `testHashEqual` (line 14)
- **Claim:** "Hashing should always produce the same result for the same input."
- **Implementation:** `assertTrue(LibOrder.hash(a) == LibOrder.hash(a))` with fuzz input `a`.
- **Verified:** Correctly tests determinism by hashing the same value twice and comparing. This is a valid property test.

### `testHashNotEqual` (line 20)
- **Claim:** "Hashing should always produce different results for different inputs."
- **Implementation:** `assertTrue(LibOrder.hash(a) != LibOrder.hash(b))` with fuzz inputs `a`, `b`.
- **Issue:** The test does not constrain `a != b`. If the fuzzer generates `a == b`, the assertion will fail. While `OrderV4` is a complex struct making collision near-impossible in 100 runs, the test does not actually verify its precondition. See finding A10-1.

---

## Findings

### A10-1: Test `testHashNotEqual` does not enforce its precondition that inputs differ (LOW)

**File:** `test/lib/LibOrder.t.sol`, line 20-22

The test claims "Hashing should always produce different results for different inputs" but does not ensure `a` and `b` are actually different. If the fuzzer happens to produce identical structs, the test incorrectly fails. The correct approach is to either `vm.assume` the inputs are different or skip the assertion when they are equal.

This is a test-only issue with no production impact, but it makes the test logically unsound -- it does not accurately test what its name describes.

---

## Summary

`LibOrder.sol` is a minimal, correct library. The single `hash` function matches its documentation exactly. The only finding is a minor test logic gap in `testHashNotEqual`.
