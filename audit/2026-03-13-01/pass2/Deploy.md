# Pass 2: Test Coverage — Deploy.sol

**Agent:** A15
**File:** `script/Deploy.sol`

## Evidence of Thorough Reading

**Contract name:** `Deploy` (inherits `Script` from forge-std)

**Functions:**
| Function | Line | Visibility |
|---|---|---|
| `deployRouter()` | 52 | internal |
| `run()` | 61 | external |

**Constants (file-level):**
| Name | Line | Type |
|---|---|---|
| `DEPLOYMENT_SUITE_ALL` | 24 | `bytes32` |
| `DEPLOYMENT_SUITE_RAINDEX` | 25 | `bytes32` |
| `DEPLOYMENT_SUITE_SUBPARSER` | 26 | `bytes32` |
| `DEPLOYMENT_SUITE_ROUTE_PROCESSOR` | 27 | `bytes32` |
| `DEPLOYMENT_SUITE_ARB` | 28 | `bytes32` |
| `ROUTE_PROCESSOR_4_CREATION_CODE` | 38 | `bytes` |
| `ROUTE_PROCESSOR_4_BYTECODE_HASH` | 41 | `bytes32` |

**Errors (file-level):**
| Name | Line |
|---|---|
| `BadRouteProcessor` | 44 |

**State variables:**
| Name | Line | Type |
|---|---|---|
| `sDepCodeHashes` | 50 | `mapping(string => mapping(address => bytes32))` |

## Test Coverage Analysis

### Search methodology

1. Searched `test/` for `Deploy`, `deployRouter`, `DEPLOYMENT_SUITE`, `ROUTE_PROCESSOR_4_CREATION_CODE`, `ROUTE_PROCESSOR_4_BYTECODE_HASH`, `BadRouteProcessor` -- none of these reference `script/Deploy.sol` or the `Deploy` contract.
2. Searched for `import.*script/Deploy` across the entire repository -- no test file imports the deploy script.
3. Checked `test/lib/deploy/LibOrderBookDeploy.t.sol` -- this tests `LibOrderBookDeploy` (deployed addresses, codehashes, etch utility) but does NOT test `script/Deploy.sol` itself.
4. Checked CI workflows: `manual-sol-artifacts.yaml` runs `rainix-sol-artifacts` which invokes `forge script script/Deploy.sol:Deploy` in a manually-triggered workflow, and `rainix.yaml` runs `rainix-sol-artifacts` as a CI task. These are dry-run exercises of the full script, not unit tests of individual code paths.

### Coverage summary

| Element | Has Unit Test | Has CI Exercise |
|---|---|---|
| `deployRouter()` | NO | Partial (only via `all` or `route-processor` suite) |
| `run()` | NO | Yes (dry-run via `rainix-sol-artifacts` in CI) |
| `DEPLOYMENT_SUITE_*` constants | NO | Indirectly |
| `ROUTE_PROCESSOR_4_CREATION_CODE` | NO | Indirectly |
| `ROUTE_PROCESSOR_4_BYTECODE_HASH` | NO | Indirectly |
| `BadRouteProcessor` error | NO | NO |
| Suite branching (`raindex`, `subparser`, `route-processor`, `arb`, `all`) | NO | Only `all` suite exercised by default CI |
| `sDepCodeHashes` mapping | NO | Indirectly |

## Findings

### A15-P2-1 [LOW] No unit test for `deployRouter()` return value on `create` failure

**Location:** Lines 52-59

The `deployRouter()` function uses inline assembly `create` to deploy the route processor. If `create` fails (e.g., out-of-gas during init code, or the creation code is corrupted), it returns `address(0)`. There is no test that verifies:
- That `deployRouter()` successfully deploys a contract with the expected `extcodehash`.
- That `deployRouter()` returning `address(0)` is caught by the downstream `BadRouteProcessor` check (or ideally an explicit zero-address check).

A dedicated test for `deployRouter()` would validate that the `ROUTE_PROCESSOR_4_CREATION_CODE` constant actually produces a contract with the `ROUTE_PROCESSOR_4_BYTECODE_HASH` codehash, which currently has no automated verification anywhere in the test suite.

### A15-P2-2 [LOW] `ROUTE_PROCESSOR_4_BYTECODE_HASH` constant is never verified against the actual deployed bytecode in tests

**Location:** Lines 38-42

The constant `ROUTE_PROCESSOR_4_BYTECODE_HASH` is hardcoded at line 41. There is no test that deploys `ROUTE_PROCESSOR_4_CREATION_CODE` and asserts the resulting `extcodehash` matches this constant. If the creation code or the hash were to drift (e.g., during a dependency update), the only way to catch it would be a failed manual deployment. This is the same class of verification that `LibOrderBookDeploy.t.sol` provides for OrderBookV6 and SubParser but is missing for the route processor.

### A15-P2-3 [LOW] No test for `BadRouteProcessor` error path

**Location:** Lines 129-135

The `BadRouteProcessor` error is defined at line 44 and reverted at line 134 when the route processor's `extcodehash` does not match the expected hash. No test exercises this revert path. A test should verify that providing a mismatched route processor address (or a zero address) triggers `BadRouteProcessor` with the correct expected vs actual hash values.

### A15-P2-4 [LOW] No test coverage for individual suite isolation (the bug from Pass 1 finding A15-1)

**Location:** Lines 61-175

Pass 1 finding A15-1 identified that running the `raindex` or `subparser` suite in isolation causes an unconditional revert at the route processor bytecode hash check (line 133). There is no test that exercises individual suite paths. A test for each suite value (`raindex`, `subparser`, `route-processor`, `arb`) would have caught this bug, since the `raindex` and `subparser` suites would revert unexpectedly.

### A15-P2-5 [INFO] Deploy script is exercised via CI dry-run but not unit-tested

The `rainix-sol-artifacts` task in `rainix.yaml` (line 30) runs the deploy script as a CI job, which provides an integration-level exercise. However, this only runs the `all` suite path (the default) and does not test individual suites, error paths, or edge cases. This is a common pattern for Foundry deploy scripts but means coverage of branching logic and error paths depends entirely on manual deployment.
