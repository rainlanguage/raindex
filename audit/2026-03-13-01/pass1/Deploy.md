# Pass 1: Security — Deploy.sol

**Agent:** A15
**File:** script/Deploy.sol

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

## Findings

### A15-1 [HIGH] Route processor bytecode hash check runs unconditionally, blocking non-route-processor suites

**Location:** Lines 129–135

The `extcodehash` check against `ROUTE_PROCESSOR_4_BYTECODE_HASH` executes unconditionally after all suite-gated blocks. When running the `raindex` or `subparser` suite alone (without setting `DEPLOY_ROUTE_PROCESSOR_4_ADDRESS`), the variable `routeProcessor` remains `address(0)` (its default from line 68). `extcodehash(address(0))` returns the empty account hash, which does not match `ROUTE_PROCESSOR_4_BYTECODE_HASH`, causing the script to revert with `BadRouteProcessor`.

This means the `raindex` and `subparser` suites cannot be run independently unless the caller also provides a valid `DEPLOY_ROUTE_PROCESSOR_4_ADDRESS` env var pointing to a pre-deployed RouteProcessor4, even though those suites have no functional dependency on the route processor.

The check (and the route processor logic on lines 122–135 in its entirety) should be gated behind the same condition used by the arb suite or route-processor suite, since only those suites need it.

### A15-2 [LOW] No explicit revert on failed `create` in `deployRouter()`

**Location:** Lines 55–58

The `create` opcode returns `address(0)` on failure (e.g., out-of-gas during init code execution, or value transfer failure). The function returns this zero address without checking it. While the downstream `extcodehash` check on line 131 would eventually catch this (since `extcodehash(address(0))` won't match the expected hash), the error message would be `BadRouteProcessor` rather than a clear indication that contract creation failed. An explicit `require(routeProcessor4 != address(0))` after the assembly block would provide a more informative failure.

### A15-3 [INFO] Private key loaded from environment variable

**Location:** Line 62

`DEPLOYMENT_KEY` is read via `vm.envUint`. This is standard practice for Foundry deployment scripts and is the expected mechanism. No issue — noted for completeness of the security surface review. The script correctly does not log or expose the key value.

### A15-4 [INFO] Unused `DEPLOYMENT_SUITE_ALL` constant if "all" suite is intended to be removed

**Location:** Lines 24, 70, 94, 123, 137

The constant `DEPLOYMENT_SUITE_ALL` is defined and actively used in four conditional branches. If the intent (per the refactor note) is to remove the "all" suite and make each suite fully self-contained, these references are stale. However, since the code currently uses the constant and the logic works for the `all` case, this is informational only — it depends on the project's intended direction.
