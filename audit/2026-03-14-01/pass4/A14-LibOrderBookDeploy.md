# A14 - LibOrderBookDeploy.sol - Pass 4 (Code Quality)

**File:** `src/lib/deploy/LibOrderBookDeploy.sol`

## Evidence

**Contract/Library:** `LibOrderBookDeploy` (library)

**Pragma:** `^0.8.25` (line 3)

**Imports (lines 5-21):**
- `Vm` from `forge-std/Vm.sol` (line 5)
- `BYTECODE_HASH`, `DEPLOYED_ADDRESS`, `RUNTIME_CODE` (as `ORDERBOOK_HASH`, `ORDERBOOK_ADDR`, `ORDERBOOK_RUNTIME_CODE`) from `../../generated/OrderBookV6.pointers.sol` (lines 7-11)
- `BYTECODE_HASH`, `DEPLOYED_ADDRESS`, `RUNTIME_CODE` (as `SUB_PARSER_HASH`, `SUB_PARSER_ADDR`, `SUB_PARSER_RUNTIME_CODE`) from `../../generated/OrderBookV6SubParser.pointers.sol` (lines 12-16)
- `BYTECODE_HASH`, `DEPLOYED_ADDRESS`, `RUNTIME_CODE` (as `ROUTE_PROCESSOR_HASH`, `ROUTE_PROCESSOR_ADDR`, `ROUTE_PROCESSOR_RUNTIME_CODE`) from `../../generated/RouteProcessor4.pointers.sol` (lines 17-21)

**Constants (lines 32-52):**
- `ORDERBOOK_DEPLOYED_ADDRESS` (line 32)
- `ORDERBOOK_DEPLOYED_CODEHASH` (line 36)
- `SUB_PARSER_DEPLOYED_ADDRESS` (line 40)
- `SUB_PARSER_DEPLOYED_CODEHASH` (line 44)
- `ROUTE_PROCESSOR_DEPLOYED_ADDRESS` (line 48)
- `ROUTE_PROCESSOR_DEPLOYED_CODEHASH` (line 52)

**Functions:**
- `etchOrderBook(Vm vm)` internal (line 58)

## Findings

### A14-1: Production library depends on forge-std/Vm.sol (MEDIUM)

**Location:** Line 5

`LibOrderBookDeploy.sol` lives under `src/lib/deploy/` (not `test/` or `script/`) and imports `Vm` from `forge-std/Vm.sol`. This is a test/script-only type -- `Vm` is the Forge cheatcode interface. Placing this in `src/` means:

1. **Leaky abstraction:** Production source code (`src/`) depends on a test framework type, blurring the boundary between production and test concerns.
2. **Submodule/library consumers** who import this project's `src/` tree will pull in a transitive dependency on `forge-std` even if they only want production contracts.
3. **Foundry convention:** The standard convention is that `src/` contains only production code. Files that depend on `Vm` belong in `test/` or `script/`.

The `etchOrderBook` function is only used in test and script contexts. This library should either be moved to `test/` or `script/`, or the `etchOrderBook` function should be extracted to a test helper, leaving only the constants in `src/`.

### A14-2: Redundant constant aliases (INFO)

**Location:** Lines 32-52

The six constants (`ORDERBOOK_DEPLOYED_ADDRESS`, `ORDERBOOK_DEPLOYED_CODEHASH`, etc.) are direct aliases of the imported values (`ORDERBOOK_ADDR`, `ORDERBOOK_HASH`, etc.) with no transformation. This is a stylistic choice that provides slightly more descriptive names at the cost of adding indirection. It is not a defect but adds cognitive overhead because a reader must trace through two layers of naming to find the actual values.

### A14-3: No commented-out code found (INFO)

No commented-out code in this file.

### A14-4: No bare `src/` imports (INFO)

All imports use relative paths (`../../generated/...`) or remapped paths (`forge-std/`). No bare `src/` imports.

## Summary

| ID | Severity | Description |
|----|----------|-------------|
| A14-1 | MEDIUM | Production `src/` library imports forge-std `Vm` -- test dependency leaks into production source tree |
| A14-2 | INFO | Constants are redundant aliases of imported values |
| A14-3 | INFO | No commented-out code |
| A14-4 | INFO | No bare `src/` imports |
