# A10 - LibOrder.sol - Pass 4 (Code Quality)

**File:** `src/lib/LibOrder.sol`
**Lines:** 19

## Evidence Inventory

### Contract/Library
- `LibOrder` (library) - line 10

### Functions
- `hash(OrderV4 memory order) -> bytes32` (internal pure) - line 16

### Imports
- `OrderV4` from `rain.raindex.interface/interface/IRaindexV6.sol` - line 5

### Constants / Types / Errors
- (none)

## Findings

### A10-1: Inconsistent Pragma Version [LOW]

`LibOrder.sol` uses `pragma solidity ^0.8.18` (line 3), while the other library files in the same directory use `^0.8.19`:
- `LibOrderBook.sol` - `^0.8.19`
- `LibOrderBookArb.sol` - `^0.8.19`
- `LibOrderBookSubParser.sol` - `^0.8.19`

The concrete contracts pin to `=0.8.25` and the deploy libs use `^0.8.25`.

The `^0.8.18` pragma in `LibOrder.sol` is the only file in `src/lib/` that uses this version. While it compiles fine alongside the rest, it is inconsistent and makes the minimum version expectation unclear. All library files in the same directory should share the same minimum version.

**Recommendation:** Bump to `^0.8.19` to match sibling library files.

---

**No other findings.** The file is concise, well-documented, uses `abi.encode` (not `abi.encodePacked`) as documented, and the single function is correct. No commented-out code, no bare `src/` imports, no leaky abstractions.
