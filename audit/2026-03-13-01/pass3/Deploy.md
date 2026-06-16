# Pass 3: Documentation — Deploy.sol

**Agent:** A15
**File:** `script/Deploy.sol`

## Evidence of Thorough Reading

**Contract name:** `Deploy` (inherits `Script` from forge-std), lines 46-176.

**Functions:**
| Function | Line | Visibility | Has Doc Comment |
|---|---|---|---|
| `deployRouter()` | 52 | internal | NO |
| `run()` | 61 | external | NO |

**File-level constants:**
| Name | Line | Type | Has Doc Comment |
|---|---|---|---|
| `DEPLOYMENT_SUITE_ALL` | 24 | `bytes32` | NO |
| `DEPLOYMENT_SUITE_RAINDEX` | 25 | `bytes32` | NO |
| `DEPLOYMENT_SUITE_SUBPARSER` | 26 | `bytes32` | NO |
| `DEPLOYMENT_SUITE_ROUTE_PROCESSOR` | 27 | `bytes32` | NO |
| `DEPLOYMENT_SUITE_ARB` | 28 | `bytes32` | NO |
| `ROUTE_PROCESSOR_4_CREATION_CODE` | 38 | `bytes` | YES (`@dev`, lines 30-37) |
| `ROUTE_PROCESSOR_4_BYTECODE_HASH` | 41 | `bytes32` | NO |

**Errors (file-level):**
| Name | Line | Has Doc Comment |
|---|---|---|
| `BadRouteProcessor(bytes32 expected, bytes32 actual)` | 44 | NO |

**State variables:**
| Name | Line | Type | Has Doc Comment |
|---|---|---|---|
| `sDepCodeHashes` | 50 | `mapping(string => mapping(address => bytes32))` | NO |

**Imports:** 20 import statements (lines 5-22), covering forge-std, OrderBookV6, SubParser, arb contracts, metadata libraries, interpreter interfaces, deployment libraries, and generated pointer files.

## Documentation Audit

### Existing documentation

1. **`ROUTE_PROCESSOR_4_CREATION_CODE`** (lines 30-37): Has a `@dev` comment explaining the source of the bytecode (SushiSwap GitHub, etherscan cross-reference) and the constructor args. This comment is accurate and helpful.

2. **Contract-level `@title` and `@notice`** (lines 46-48): Present but contain inaccuracies (see findings below).

### Missing documentation

The following elements have no NatSpec or doc comments at all:
- All five `DEPLOYMENT_SUITE_*` constants (lines 24-28)
- `ROUTE_PROCESSOR_4_BYTECODE_HASH` (line 41)
- `BadRouteProcessor` error (line 44)
- `sDepCodeHashes` state variable (line 50)
- `deployRouter()` function (line 52)
- `run()` function (line 61)

## Findings

### A15-P3-1 [LOW] Contract `@notice` references deprecated "mumbai" testnet

**Location:** Lines 47-48

The contract-level NatSpec says:

```
/// @notice A script that deploys all contracts. This is intended to be run on
/// every commit by CI to a testnet such as mumbai.
```

Polygon Mumbai testnet was deprecated in April 2024. This reference is stale and misleading. Additionally, the description "deploys all contracts" is imprecise since the script supports selective deployment via the `DEPLOYMENT_SUITE` env var and may deploy only a subset of contracts.

### A15-P3-2 [LOW] `deployRouter()` has no documentation

**Location:** Line 52

The `deployRouter()` function has no NatSpec at all. It uses inline assembly to deploy a contract from `ROUTE_PROCESSOR_4_CREATION_CODE` via `create`. A developer reading this function needs to understand:
- What contract it deploys (SushiSwap RouteProcessor4)
- That it uses `create` (not `create2`), so the address is nondeterministic
- What happens on failure (returns `address(0)` -- as noted in Pass 1 finding A15-2)
- What the return value represents

### A15-P3-3 [LOW] `run()` function has no documentation describing env var interface

**Location:** Line 61

The `run()` function is the main entry point for the deployment script. It reads three environment variables (`DEPLOYMENT_KEY`, `DEPLOYMENT_SUITE`, `DEPLOY_ROUTE_PROCESSOR_4_ADDRESS` and conditionally `DEPLOY_RAINDEX_ADDRESS`) but none of these are documented. There is no `@notice` or `@dev` explaining:
- The purpose and expected values for each env var
- The valid suite values (`all`, `raindex`, `subparser`, `route-processor`, `arb`)
- The deployment flow and what each suite deploys
- The dependency between suites (e.g., `arb` requires `raindex` address)

This is the primary user-facing function of the script, and its lack of documentation forces operators to read the full implementation to understand how to use it.

### A15-P3-4 [LOW] Five `DEPLOYMENT_SUITE_*` constants have no documentation

**Location:** Lines 24-28

The five suite selector constants are undocumented. While the naming convention is somewhat self-explanatory, there is no doc comment explaining:
- What each suite deploys
- How they relate to each other (e.g., `all` is a superset)
- That the values are `keccak256` hashes of the corresponding string identifiers

### A15-P3-5 [INFO] `ROUTE_PROCESSOR_4_BYTECODE_HASH` lacks provenance documentation

**Location:** Line 41-42

The `ROUTE_PROCESSOR_4_CREATION_CODE` constant has thorough provenance documentation (lines 30-37) citing the SushiSwap GitHub and etherscan. The corresponding `ROUTE_PROCESSOR_4_BYTECODE_HASH` has no such documentation. It would be helpful to note that this is the `extcodehash` of the runtime bytecode produced by deploying `ROUTE_PROCESSOR_4_CREATION_CODE`, and ideally how it was derived or verified.

### A15-P3-6 [INFO] `BadRouteProcessor` error has no NatSpec

**Location:** Line 44

The error `BadRouteProcessor(bytes32 expected, bytes32 actual)` has no documentation. The parameter names are reasonably self-descriptive, but a brief `@dev` or inline comment explaining when this error is triggered would be helpful.

### A15-P3-7 [INFO] `sDepCodeHashes` state variable has no documentation

**Location:** Line 50

The `sDepCodeHashes` mapping has no doc comment. Its purpose (tracking dependency code hashes for `LibRainDeploy.deployAndBroadcast`) is not explained.

### A15-P3-8 [INFO] README.md does not mention deployment

**Location:** `/README.md`

The top-level README describes the repository structure, local development setup, and legal information, but contains no section on deployment. There is no mention of `script/Deploy.sol`, the `DEPLOYMENT_SUITE` env var, or how to run the deploy script. While deployment instructions may be considered internal/CI-only knowledge, the README's "Setup for local development" section would be a natural place to at least reference the deploy script's existence and point to further documentation.
