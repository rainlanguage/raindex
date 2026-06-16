# A15: Deploy.sol - Pass 1 (Security)

## Evidence of Thorough Reading

**File:** `script/Deploy.sol` (143 lines)

### Contract
- `Deploy is Script` (line 39)

### Functions
- `run()` - line 46

### State variables
- `sDepCodeHashes` - `mapping(string => mapping(address => bytes32))` (line 42)

### Constants (file-level)
- `DEPLOYMENT_SUITE_RAINDEX` = keccak256("raindex") (line 27)
- `DEPLOYMENT_SUITE_SUBPARSER` = keccak256("subparser") (line 29)
- `DEPLOYMENT_SUITE_ROUTE_PROCESSOR` = keccak256("route-processor") (line 31)
- `DEPLOYMENT_SUITE_ARB` = keccak256("arb") (line 33)

### Imports
- `Script`, `console2` from forge-std
- `EvaluableV4`, `SignedContextV1` from IInterpreterCallerV4
- `TaskV2` from IRaindexV6
- `OrderBookV6SubParser` from concrete/parser
- `GenericPoolOrderBookV6ArbOrderTaker` from concrete/arb
- `RouteProcessorOrderBookV6ArbOrderTaker` from concrete/arb
- `GenericPoolOrderBookV6FlashBorrower` from concrete/arb
- `OrderBookV6ArbConfig` from abstract/OrderBookV6ArbCommon
- `IMetaBoardV1_2` from rain.metadata
- `LibDescribedByMeta` from rain.metadata
- `LibMetaBoardDeploy` from rain.metadata
- `LibDecimalFloatDeploy` from rain.math.float
- `LibTOFUTokenDecimals` from rain.tofu.erc20-decimals
- `IInterpreterStoreV3` from rain.interpreter.interface
- `IInterpreterV4` from rain.interpreter.interface
- `LibRainDeploy` from rain.deploy
- `LibOrderBookDeploy` from src/lib/deploy
- `CREATION_CODE` as ORDERBOOK_CREATION_CODE from OrderBookV6.pointers.sol
- `CREATION_CODE` as SUB_PARSER_CREATION_CODE from OrderBookV6SubParser.pointers.sol
- `ROUTE_PROCESSOR_4_CREATION_CODE` from LibRouteProcessor4CreationCode

## Findings

### A15-1: String revert used for unknown deployment suite (LOW)

**Location:** `script/Deploy.sol`, line 140

**Description:** The `run()` function uses `revert("Unknown deployment suite")` with a string message. The project convention (per CLAUDE.md in the interface repos and general codebase style) is to use custom errors exclusively and avoid string reverts. While this is a Foundry deployment script (not production runtime code), it is still Solidity source in the repository and should follow the project's coding standards for consistency and gas efficiency.

**Impact:** Minimal -- this code only runs in deployment scripts, not on-chain in production. However, it deviates from the project's "all reverts use custom errors" convention.
