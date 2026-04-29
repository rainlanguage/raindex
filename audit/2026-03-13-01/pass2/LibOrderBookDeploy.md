# Pass 2: Test Coverage -- LibOrderBookDeploy.sol

**Agent:** A14
**File:** `src/lib/deploy/LibOrderBookDeploy.sol`
**Test file:** `test/lib/deploy/LibOrderBookDeploy.t.sol`

## Evidence of Thorough Reading

**Library name:** `LibOrderBookDeploy`

**Functions (with line numbers):**
- `etchOrderBook(Vm vm)` -- line 45 (internal). Etches runtime bytecode for both OrderBookV6 and OrderBookV6SubParser at their expected deterministic addresses, skipping any contract whose codehash already matches.

**Constants (with line numbers):**
- `ORDERBOOK_DEPLOYED_ADDRESS` (address) -- line 27
- `ORDERBOOK_DEPLOYED_CODEHASH` (bytes32) -- line 31
- `SUB_PARSER_DEPLOYED_ADDRESS` (address) -- line 35
- `SUB_PARSER_DEPLOYED_CODEHASH` (bytes32) -- line 39

**Types/errors/custom types:** None.

**Imports:**
- `Vm` from `forge-std/Vm.sol` (line 5)
- `BYTECODE_HASH`, `DEPLOYED_ADDRESS`, `RUNTIME_CODE` from `OrderBookV6.pointers.sol` (lines 7-11)
- `BYTECODE_HASH`, `DEPLOYED_ADDRESS`, `RUNTIME_CODE` from `OrderBookV6SubParser.pointers.sol` (lines 12-16)

## Test Inventory

The test file `test/lib/deploy/LibOrderBookDeploy.t.sol` contains 10 tests in `LibOrderBookDeployTest`:

| # | Test | What it covers |
|---|------|----------------|
| 1 | `testDeployAddressOrderBook` | Zoltu deploy of OrderBookV6 produces expected address and codehash |
| 2 | `testDeployAddressSubParser` | Zoltu deploy of SubParser produces expected address and codehash |
| 3 | `testExpectedCodeHashOrderBook` | Fresh `new OrderBookV6()` codehash matches constant |
| 4 | `testExpectedCodeHashSubParser` | Fresh `new OrderBookV6SubParser()` codehash matches constant |
| 5 | `testCreationCodeOrderBook` | Generated creation code constant matches compiler output |
| 6 | `testCreationCodeSubParser` | Generated creation code constant matches compiler output |
| 7 | `testRuntimeCodeOrderBook` | Generated runtime code constant matches deployed bytecode |
| 8 | `testRuntimeCodeSubParser` | Generated runtime code constant matches deployed bytecode |
| 9 | `testGeneratedDeployedAddressOrderBook` | Generated address constant matches library constant |
| 10 | `testGeneratedDeployedAddressSubParser` | Generated address constant matches library constant |
| 11 | `testEtchOrderBook` | After `etchOrderBook`, both addresses have correct codehash |
| 12 | `testEtchOrderBookIdempotent` | Calling `etchOrderBook` twice is idempotent |

## Coverage Analysis

### Constants coverage

All four constants (`ORDERBOOK_DEPLOYED_ADDRESS`, `ORDERBOOK_DEPLOYED_CODEHASH`, `SUB_PARSER_DEPLOYED_ADDRESS`, `SUB_PARSER_DEPLOYED_CODEHASH`) are thoroughly verified:
- Against Zoltu factory deployments (tests 1-2)
- Against direct `new` deployments (tests 3-4)
- Against generated pointer file values (tests 9-10)
- Creation code and runtime code constants also cross-checked (tests 5-8)

This is strong multi-angle validation that the generated constants are correct.

### Function coverage: `etchOrderBook`

The function has two independent conditional branches (lines 46-48 and lines 49-51), creating four possible execution paths:

| Path | OB needs etch | SP needs etch | Tested? |
|------|--------------|---------------|---------|
| A | Yes | Yes | Yes -- `testEtchOrderBook` (fresh state, both addresses empty) |
| B | No | No | Yes -- `testEtchOrderBookIdempotent` (second call) |
| C | Yes | No | No |
| D | No | Yes | No |

Paths C and D (partial etch) are not directly tested. In path C, OrderBook is missing but SubParser is already etched. In path D, the reverse.

### Integration coverage

The library is also exercised indirectly in test utilities:
- `OrderBookV6ExternalMockTest` constructor calls `LibOrderBookDeploy.etchOrderBook(vm)` (line 51)
- `OrderBookV6ExternalRealTest` and `OrderBookV6SubParserContextTest` also use it
- `Deploy.sol` script references the constants for production deployments

## Findings

### INFO-A14-1: Partial etch paths (C and D) are untested

**Severity:** INFO
**Location:** `etchOrderBook` lines 46-51

The two `if` guards in `etchOrderBook` are independent, but tests only exercise both-need-etch (path A) and neither-needs-etch (path B). Paths where only one contract needs etching are not directly tested.

**Impact:** Negligible. The two `if` blocks are structurally independent -- they share no state, no variables, and no control flow coupling. Each branch is a simple comparison + `vm.etch` call. The combined paths (A and B) already exercise both the "etch" and "skip" logic for each branch, just not in the mixed combination. There is no realistic scenario where testing paths C or D would reveal a bug that A and B do not.

**Recommendation:** No action required. Adding explicit tests for partial etch would marginally improve formal branch coverage but provides no practical security benefit given the simplicity of the code.

## Summary

Test coverage for `LibOrderBookDeploy.sol` is thorough and well-structured. The tests take a commendable multi-angle verification approach: every constant is validated against Zoltu factory deployment, direct deployment, and generated pointer files. The `etchOrderBook` function is tested for both the initial etch case and idempotency. The only untested paths are partial-etch combinations, which are structurally independent branches with no interaction effects, making this an informational observation only.

No LOW+ findings. No fix files required.
