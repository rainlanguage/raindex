# A14: LibOrderBookDeploy.sol - Pass 1 (Security)

## Evidence of Thorough Reading

**File:** `src/lib/deploy/LibOrderBookDeploy.sol` (69 lines)

### Library
- `LibOrderBookDeploy` (line 29)

### Functions
- `etchOrderBook(Vm vm)` - line 58

### Constants
- `ORDERBOOK_DEPLOYED_ADDRESS` = ORDERBOOK_ADDR (line 32)
- `ORDERBOOK_DEPLOYED_CODEHASH` = ORDERBOOK_HASH (line 36)
- `SUB_PARSER_DEPLOYED_ADDRESS` = SUB_PARSER_ADDR (line 40)
- `SUB_PARSER_DEPLOYED_CODEHASH` = SUB_PARSER_HASH (line 44)
- `ROUTE_PROCESSOR_DEPLOYED_ADDRESS` = ROUTE_PROCESSOR_ADDR (line 48)
- `ROUTE_PROCESSOR_DEPLOYED_CODEHASH` = ROUTE_PROCESSOR_HASH (line 52)

### Imports
- `Vm` from forge-std
- `BYTECODE_HASH`, `DEPLOYED_ADDRESS`, `RUNTIME_CODE` as ORDERBOOK_* from OrderBookV6.pointers.sol
- `BYTECODE_HASH`, `DEPLOYED_ADDRESS`, `RUNTIME_CODE` as SUB_PARSER_* from OrderBookV6SubParser.pointers.sol
- `BYTECODE_HASH`, `DEPLOYED_ADDRESS`, `RUNTIME_CODE` as ROUTE_PROCESSOR_* from RouteProcessor4.pointers.sol

## Findings

No security findings. Analysis:

- **Test-only library**: This library uses `Vm` (Forge cheatcodes), meaning it can only run in a Foundry test/script context, not in production. The `vm.etch` cheatcode is a test primitive.
- **Codehash guard**: Each `etch` call is guarded by a codehash comparison (lines 59, 62, 65), preventing unnecessary etching when the contract is already correctly deployed. This is idempotent and safe.
- **No external calls to untrusted contracts**: All interactions are with the Forge VM cheatcode interface.
- **No assembly, no string reverts, no state mutations**: Pure deployment utility.
