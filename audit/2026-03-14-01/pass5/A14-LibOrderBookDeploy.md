# A14 - Pass 5: Correctness / Intent Verification
## File: `src/lib/deploy/LibOrderBookDeploy.sol`

### Evidence: Contract Inventory

**Library**: `LibOrderBookDeploy`

**Constants** (lines 32-52):
- `ORDERBOOK_DEPLOYED_ADDRESS` (line 32) -- aliased from generated `ORDERBOOK_ADDR`
- `ORDERBOOK_DEPLOYED_CODEHASH` (line 36) -- aliased from generated `ORDERBOOK_HASH`
- `SUB_PARSER_DEPLOYED_ADDRESS` (line 40) -- aliased from generated `SUB_PARSER_ADDR`
- `SUB_PARSER_DEPLOYED_CODEHASH` (line 44) -- aliased from generated `SUB_PARSER_HASH`
- `ROUTE_PROCESSOR_DEPLOYED_ADDRESS` (line 48) -- aliased from generated `ROUTE_PROCESSOR_ADDR`
- `ROUTE_PROCESSOR_DEPLOYED_CODEHASH` (line 52) -- aliased from generated `ROUTE_PROCESSOR_HASH`

**Functions** (lines 58-68):
- `etchOrderBook(Vm vm)` (line 58) -- etches runtime bytecode of OrderBook, SubParser, and RouteProcessor4

### Verification: NatSpec vs Implementation

#### Library-level NatSpec (lines 23-28)
NatSpec: "A library containing the deployed address and code hash of the OrderBook contracts when deployed with the rain standard zoltu deployer."

Implementation: The library also contains address/hash constants for RouteProcessor4, which is not an "OrderBook contract" -- it is a Sushi RouteProcessor. The NatSpec is slightly incomplete but not materially misleading, as RouteProcessor4 is part of the OrderBook deployment.

#### `etchOrderBook` NatSpec (lines 54-57)
NatSpec: "Etches the runtime bytecode of the orderbook and sub parser at their expected deterministic addresses. Skips any contract whose codehash already matches."

Implementation: The function etches **three** contracts -- OrderBook, SubParser, **and RouteProcessor4** (lines 65-67). The NatSpec omits RouteProcessor4.

### Verification: `etchOrderBook` logic

The skip logic is correct for each contract:
1. If `ORDERBOOK_DEPLOYED_CODEHASH != ORDERBOOK_DEPLOYED_ADDRESS.codehash` -> etch OrderBook. Correct: skip if already deployed with matching code.
2. Same pattern for SubParser. Correct.
3. Same pattern for RouteProcessor4. Correct.

The `.codehash` of an address with no code is `bytes32(0)`, which will not equal the expected hash, so the etch proceeds on fresh addresses. Correct.

### Verification: Tests vs Claims

**`LibOrderBookDeployTest`** (`test/lib/deploy/LibOrderBookDeploy.t.sol`):

| Test | Claim | Exercises? |
|---|---|---|
| `testDeployAddressOrderBook` | "Deploying OrderBookV6 via Zoltu MUST produce the expected address and codehash" | Yes -- deploys via Zoltu factory and checks both address and codehash |
| `testDeployAddressSubParser` | Same for SubParser | Yes |
| `testExpectedCodeHashOrderBook` | "The codehash of a freshly deployed OrderBookV6 MUST match the expected codehash constant" | Yes -- deploys with `new` and checks codehash |
| `testExpectedCodeHashSubParser` | Same for SubParser | Yes |
| `testCreationCodeOrderBook` | "The precompiled creation code constant MUST match the compiler's creation code" | Yes |
| `testCreationCodeSubParser` | Same for SubParser | Yes |
| `testRuntimeCodeOrderBook` | "The precompiled runtime code constant MUST match the deployed runtime bytecode" | Yes |
| `testRuntimeCodeSubParser` | Same for SubParser | Yes |
| `testGeneratedDeployedAddressOrderBook` | "The generated deployed address MUST match the deploy library constant" | Yes |
| `testGeneratedDeployedAddressSubParser` | Same for SubParser | Yes |
| `testEtchOrderBook` | "After calling etchOrderBook, both contracts MUST have the expected codehash" | Partially -- only checks OrderBook and SubParser, not RouteProcessor4 |
| `testEtchOrderBookIdempotent` | "Calling etchOrderBook twice MUST be idempotent" | Partially -- same omission |

**`LibOrderBookDeployProdTest`** (`test/lib/deploy/LibOrderBookDeployProd.t.sol`):
- `_checkAllContracts()` checks code exists and codehash matches for OrderBook and SubParser only.
- Does not verify RouteProcessor4 on any production network.

### Verification: Constants match generated pointers

All six constants are direct aliases of the imported generated values. No transformation or transcription errors are possible since they are compile-time assignments.

### Findings

#### A14-1: `etchOrderBook` NatSpec omits RouteProcessor4
**Severity**: LOW

**Location**: `src/lib/deploy/LibOrderBookDeploy.sol`, lines 54-57

**Description**: The `@notice` for `etchOrderBook` says "Etches the runtime bytecode of the orderbook and sub parser" but the function also etches `ROUTE_PROCESSOR_DEPLOYED_ADDRESS` with `ROUTE_PROCESSOR_RUNTIME_CODE` (lines 65-67). The NatSpec should mention all three contracts.

**Impact**: Developers relying on the NatSpec alone will not know that RouteProcessor4 is also etched, which could cause confusion about test setup side effects.

#### A14-2: `testEtchOrderBook` and `testEtchOrderBookIdempotent` do not verify RouteProcessor4
**Severity**: LOW

**Location**: `test/lib/deploy/LibOrderBookDeploy.t.sol`, lines 100-119

**Description**: Both tests call `LibOrderBookDeploy.etchOrderBook(vm)` but only assert the codehashes for `ORDERBOOK_DEPLOYED_ADDRESS` and `SUB_PARSER_DEPLOYED_ADDRESS`. They do not assert that `ROUTE_PROCESSOR_DEPLOYED_ADDRESS.codehash == ROUTE_PROCESSOR_DEPLOYED_CODEHASH`. The tests do not fully verify the behavior of the function they claim to test.

**Impact**: If the RouteProcessor4 etching broke (e.g., wrong runtime code constant), these tests would not catch it. The `LibRouteProcessor4CreationCode.t.sol` test partially covers this by verifying the codehash of a freshly deployed instance, but does not test the `etchOrderBook` path specifically.

#### A14-3: `LibOrderBookDeployProdTest` does not verify RouteProcessor4 on production networks
**Severity**: LOW

**Location**: `test/lib/deploy/LibOrderBookDeployProd.t.sol`, lines 13-21

**Description**: `_checkAllContracts()` checks existence and codehash for OrderBook and SubParser on each forked network but does not verify RouteProcessor4. The library exposes `ROUTE_PROCESSOR_DEPLOYED_ADDRESS` and `ROUTE_PROCESSOR_DEPLOYED_CODEHASH` as public constants, implying they should be verified in production the same way.

**Impact**: If RouteProcessor4 is not deployed on a target network, or is deployed with different bytecode, the production test suite will not catch it.
