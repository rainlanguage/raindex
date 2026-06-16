# Pass 3: Documentation -- A14 LibOrderBookDeploy

**File:** `src/lib/deploy/LibOrderBookDeploy.sol`

## Evidence of Reading

**Library:** `LibOrderBookDeploy` (line 29)

**Library-level NatSpec (lines 23-28):**
- `@title LibOrderBookDeploy`
- `@notice` describes that the library holds deployed addresses and code hashes for OrderBook contracts deployed via the rain standard zoltu deployer, enabling idempotent deployments.

**Constants (all `internal` visibility via default for library constants):**
1. `ORDERBOOK_DEPLOYED_ADDRESS` (line 32) -- documented lines 30-31
2. `ORDERBOOK_DEPLOYED_CODEHASH` (line 36) -- documented lines 34-35
3. `SUB_PARSER_DEPLOYED_ADDRESS` (line 40) -- documented lines 38-39
4. `SUB_PARSER_DEPLOYED_CODEHASH` (line 44) -- documented lines 42-43
5. `ROUTE_PROCESSOR_DEPLOYED_ADDRESS` (line 48) -- documented lines 46-47
6. `ROUTE_PROCESSOR_DEPLOYED_CODEHASH` (line 52) -- documented lines 50-51

**Functions:**
1. `etchOrderBook(Vm vm)` -- line 58, `internal`
   - `@notice` on lines 54-56: "Etches the runtime bytecode of the orderbook and sub parser at their expected deterministic addresses. Skips any contract whose codehash already matches."
   - `@param vm` on line 57: "The Forge `Vm` cheatcode interface."

**Imports (lines 7-21):**
- `BYTECODE_HASH`, `DEPLOYED_ADDRESS`, `RUNTIME_CODE` from `OrderBookV6.pointers.sol`
- `BYTECODE_HASH`, `DEPLOYED_ADDRESS`, `RUNTIME_CODE` from `OrderBookV6SubParser.pointers.sol`
- `BYTECODE_HASH`, `DEPLOYED_ADDRESS`, `RUNTIME_CODE` from `RouteProcessor4.pointers.sol`

## Findings

### A14-1: `etchOrderBook` NatSpec mentions "orderbook and sub parser" but also etches RouteProcessor4 (LOW)

**Severity:** LOW

The `@notice` on line 54-56 says: "Etches the runtime bytecode of the orderbook and sub parser at their expected deterministic addresses." However, the function body (lines 65-67) also etches `ROUTE_PROCESSOR_DEPLOYED_ADDRESS` with `ROUTE_PROCESSOR_RUNTIME_CODE`. The doc omits the RouteProcessor4 contract.

**Location:** Lines 54-56 vs lines 65-67.

### A14-2: No `@return` documentation on `etchOrderBook` (INFO)

**Severity:** INFO

The function returns nothing (`void`), so no `@return` is needed. This is correct and not a finding.

No further findings. The documentation is well-structured with accurate NatSpec on all constants and the single function.
