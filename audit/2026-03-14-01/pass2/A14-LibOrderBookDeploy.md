# Pass 2: Test Coverage -- A14 LibOrderBookDeploy

**File:** src/lib/deploy/LibOrderBookDeploy.sol

## Evidence of Reading

### Imports (lines 5-21)
- `Vm` from forge-std
- `BYTECODE_HASH`, `DEPLOYED_ADDRESS`, `RUNTIME_CODE` from `OrderBookV6.pointers.sol` (aliased as `ORDERBOOK_HASH`, `ORDERBOOK_ADDR`, `ORDERBOOK_RUNTIME_CODE`)
- `BYTECODE_HASH`, `DEPLOYED_ADDRESS`, `RUNTIME_CODE` from `OrderBookV6SubParser.pointers.sol` (aliased as `SUB_PARSER_HASH`, `SUB_PARSER_ADDR`, `SUB_PARSER_RUNTIME_CODE`)
- `BYTECODE_HASH`, `DEPLOYED_ADDRESS`, `RUNTIME_CODE` from `RouteProcessor4.pointers.sol` (aliased as `ROUTE_PROCESSOR_HASH`, `ROUTE_PROCESSOR_ADDR`, `ROUTE_PROCESSOR_RUNTIME_CODE`)

### Library: `LibOrderBookDeploy` (lines 29-69)

#### Constants (lines 32-52)
- `ORDERBOOK_DEPLOYED_ADDRESS` (line 32) = `ORDERBOOK_ADDR`
- `ORDERBOOK_DEPLOYED_CODEHASH` (line 36) = `ORDERBOOK_HASH`
- `SUB_PARSER_DEPLOYED_ADDRESS` (line 40) = `SUB_PARSER_ADDR`
- `SUB_PARSER_DEPLOYED_CODEHASH` (line 44) = `SUB_PARSER_HASH`
- `ROUTE_PROCESSOR_DEPLOYED_ADDRESS` (line 48) = `ROUTE_PROCESSOR_ADDR`
- `ROUTE_PROCESSOR_DEPLOYED_CODEHASH` (line 52) = `ROUTE_PROCESSOR_HASH`

#### Function: `etchOrderBook(Vm vm)` (lines 58-68)
- `internal` function, takes the Forge `Vm` cheatcode interface
- Line 59: If `ORDERBOOK_DEPLOYED_CODEHASH != ORDERBOOK_DEPLOYED_ADDRESS.codehash`, etches `ORDERBOOK_RUNTIME_CODE` at `ORDERBOOK_DEPLOYED_ADDRESS`
- Line 62: If `SUB_PARSER_DEPLOYED_CODEHASH != SUB_PARSER_DEPLOYED_ADDRESS.codehash`, etches `SUB_PARSER_RUNTIME_CODE` at `SUB_PARSER_DEPLOYED_ADDRESS`
- Line 65: If `ROUTE_PROCESSOR_DEPLOYED_CODEHASH != ROUTE_PROCESSOR_DEPLOYED_ADDRESS.codehash`, etches `ROUTE_PROCESSOR_RUNTIME_CODE` at `ROUTE_PROCESSOR_DEPLOYED_ADDRESS`

### No errors, events, or custom types defined in this file.

## Test Files Reviewed

### `test/lib/deploy/LibOrderBookDeploy.t.sol` (120 lines)
10 test functions in `LibOrderBookDeployTest`:
- `testDeployAddressOrderBook` (line 24): Deploys OrderBookV6 via Zoltu, asserts address and codehash match library constants
- `testDeployAddressSubParser` (line 36): Deploys SubParser via Zoltu, asserts address and codehash match library constants
- `testExpectedCodeHashOrderBook` (line 48): `new OrderBookV6()` codehash matches `ORDERBOOK_DEPLOYED_CODEHASH`
- `testExpectedCodeHashSubParser` (line 55): `new OrderBookV6SubParser()` codehash matches `SUB_PARSER_DEPLOYED_CODEHASH`
- `testCreationCodeOrderBook` (line 62): Precompiled creation code matches compiler output
- `testCreationCodeSubParser` (line 68): Precompiled creation code matches compiler output
- `testRuntimeCodeOrderBook` (line 74): Precompiled runtime code matches deployed bytecode
- `testRuntimeCodeSubParser` (line 81): Precompiled runtime code matches deployed bytecode
- `testGeneratedDeployedAddressOrderBook` (line 88): Generated address matches library constant
- `testGeneratedDeployedAddressSubParser` (line 94): Generated address matches library constant
- `testEtchOrderBook` (line 100): Calls `etchOrderBook`, asserts codehash for OrderBook and SubParser only
- `testEtchOrderBookIdempotent` (line 111): Double-calls `etchOrderBook`, asserts codehash for OrderBook and SubParser only

### `test/lib/deploy/LibOrderBookDeployProd.t.sol` (52 lines)
5 fork tests in `LibOrderBookDeployProdTest`:
- `testProdDeployArbitrum` / `testProdDeployBase` / `testProdDeployBaseSepolia` / `testProdDeployFlare` / `testProdDeployPolygon`
- Each forks the network and calls `_checkAllContracts()` which verifies code.length > 0 and codehash for OrderBook and SubParser only

### Indirect usage (not direct unit tests of this library)
- `test/util/abstract/ArbTest.sol`: Uses `LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS` to etch mock OB code
- `test/util/abstract/OrderBookV6ExternalMockTest.sol`: Calls `etchOrderBook(vm)` and uses `ORDERBOOK_DEPLOYED_ADDRESS`
- `test/util/abstract/OrderBookV6ExternalRealTest.sol`: Calls `etchOrderBook(vm)` and uses `ORDERBOOK_DEPLOYED_ADDRESS` and `SUB_PARSER_DEPLOYED_ADDRESS`
- `test/util/abstract/OrderBookV6SubParserContextTest.sol`: Calls `etchOrderBook(vm)` and uses `SUB_PARSER_DEPLOYED_ADDRESS`

### Related test
- `test/lib/deploy/LibRouteProcessor4CreationCode.t.sol`: Tests that deploying `ROUTE_PROCESSOR_4_CREATION_CODE` yields the known Sushi codehash. Does NOT reference `LibOrderBookDeploy` or verify any of its RouteProcessor constants.

## Findings

### A14-1: RouteProcessor constants have no unit test coverage [MEDIUM]

**Severity:** MEDIUM

**Details:** `LibOrderBookDeploy` defines 3 pairs of address/codehash constants. The test file `LibOrderBookDeploy.t.sol` has comprehensive tests for OrderBook and SubParser constants (Zoltu deploy address, codehash, creation code, runtime code, generated address), but zero tests for the RouteProcessor pair:
- No test that `ROUTE_PROCESSOR_DEPLOYED_ADDRESS` matches the generated `ROUTE_PROCESSOR_ADDR`
- No test that `ROUTE_PROCESSOR_DEPLOYED_CODEHASH` matches the generated `ROUTE_PROCESSOR_HASH`
- No test that deploying RouteProcessor4 via Zoltu produces the expected address
- No test that the deployed runtime codehash matches `ROUTE_PROCESSOR_DEPLOYED_CODEHASH`

The existing `LibRouteProcessor4CreationCode.t.sol` test only verifies the creation code produces a known codehash via `create`. It does not test the Zoltu deploy path or verify any constant from `LibOrderBookDeploy`.

If the generated pointer file for RouteProcessor4 were to drift or contain an error, no test would catch it.

### A14-2: `etchOrderBook` RouteProcessor branch not verified [MEDIUM]

**Severity:** MEDIUM

**Details:** The `etchOrderBook` function (lines 58-68) etches 3 contracts. Both `testEtchOrderBook` and `testEtchOrderBookIdempotent` only assert the codehash of OrderBook and SubParser after calling `etchOrderBook`. Neither test asserts that `ROUTE_PROCESSOR_DEPLOYED_ADDRESS.codehash == ROUTE_PROCESSOR_DEPLOYED_CODEHASH` after etching. The RouteProcessor etch path (lines 65-67) has zero test verification.

This means a bug in the RouteProcessor etch logic (e.g., wrong runtime code, wrong address) would go undetected.

### A14-3: Prod fork tests do not verify RouteProcessor deployment [LOW]

**Severity:** LOW

**Details:** `LibOrderBookDeployProd.t.sol` forks 5 networks and calls `_checkAllContracts()`, which only verifies OrderBook and SubParser are deployed with correct codehashes. It does not check whether RouteProcessor4 is deployed at `ROUTE_PROCESSOR_DEPLOYED_ADDRESS` on any network. If the RouteProcessor were not deployed on a network where it is expected, no test would detect this.
