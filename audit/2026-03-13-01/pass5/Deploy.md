# Pass 5: Correctness -- Deploy.sol

**Agent:** A15
**File:** `script/Deploy.sol`

## Evidence of Thorough Reading

**Contract name:** `Deploy` (inherits `Script` from forge-std), lines 46-176.

**Line-by-line verification inventory:**

| Element | Lines | Verified |
|---|---|---|
| `DEPLOYMENT_SUITE_ALL = keccak256("all")` | 24 | String matches constant name suffix |
| `DEPLOYMENT_SUITE_RAINDEX = keccak256("raindex")` | 25 | String matches constant name suffix |
| `DEPLOYMENT_SUITE_SUBPARSER = keccak256("subparser")` | 26 | String matches constant name suffix |
| `DEPLOYMENT_SUITE_ROUTE_PROCESSOR = keccak256("route-processor")` | 27 | String matches constant name suffix |
| `DEPLOYMENT_SUITE_ARB = keccak256("arb")` | 28 | String matches constant name suffix |
| `ROUTE_PROCESSOR_4_CREATION_CODE` | 38-39 | Hex literal present, constructor args decoded below |
| `ROUTE_PROCESSOR_4_BYTECODE_HASH` | 41-42 | Compared against `extcodehash` at runtime |
| `BadRouteProcessor(bytes32 expected, bytes32 actual)` | 44 | Error signature matches usage at line 134 |
| `sDepCodeHashes` | 50 | Used as pass-through to `LibRainDeploy.deployAndBroadcast` |
| `deployRouter()` | 52-59 | Assembly verified (see below) |
| `run()` | 61-175 | Suite branching verified (see below) |

**Assembly verification (`deployRouter()`, line 55-57):**
- `create(0, add(routeProcessor4Code, 0x20), mload(routeProcessor4Code))` is the standard pattern for deploying from a `bytes memory` variable.
- `value = 0`: no ETH sent with deployment. Correct for RouteProcessor4 which does not require constructor value.
- `offset = add(routeProcessor4Code, 0x20)`: skips the 32-byte ABI length prefix to point at actual bytecode. Correct.
- `size = mload(routeProcessor4Code)`: reads the length from the first 32 bytes of the memory pointer. Correct.
- `"memory-safe"` annotation: the block only reads memory and writes to a stack variable. Correct.

**Constructor args verification (trailing bytes of `ROUTE_PROCESSOR_4_CREATION_CODE`):**
The last 96 bytes (3 words) of the hex literal decode as ABI-encoded constructor arguments for `constructor(address _bentoBox, address[] memory _privilegedUsers)`:
- Word 0: `0x0...0` = `address(0)` -- bentoBox parameter (no bento). Matches comment on line 36.
- Word 1: `0x0...40` = 64 -- byte offset to the dynamic array data. Correct ABI encoding.
- Word 2: `0x0...0` = 0 -- array length (empty privileged users array). Matches comment on line 37 ("no owner addresses").

**DEPLOYMENT_SUITE_* constant/string consistency:**
All five constants use `keccak256("...")` where the string inside the hash exactly matches the lowercase, hyphenated suffix of the constant name (e.g., `DEPLOYMENT_SUITE_ROUTE_PROCESSOR` hashes `"route-processor"`, not `"route_processor"`). The default `suiteString` at line 64 is `"all"`, which matches `DEPLOYMENT_SUITE_ALL`. Verified correct.

**Suite branching logic (line-by-line flow analysis):**

| Suite value | Raindex block (L70) | Subparser block (L94) | Route processor deploy (L122) | Bytecode check (L129) | Arb block (L137) |
|---|---|---|---|---|---|
| `"all"` | YES | YES | YES (if env not set) | YES | YES |
| `"raindex"` | YES | NO | NO | YES (BUG) | NO |
| `"subparser"` | NO | YES | NO | YES (BUG) | NO |
| `"route-processor"` | NO | NO | YES | YES | NO |
| `"arb"` | NO | NO | NO | YES | YES |

**`DEPLOYMENT_SUITE_ALL` completeness check:**
When `suite == "all"`, every gated block's condition includes `suite == DEPLOYMENT_SUITE_ALL`:
- Line 70: `suite == DEPLOYMENT_SUITE_RAINDEX || suite == DEPLOYMENT_SUITE_ALL` -- YES
- Line 94: `suite == DEPLOYMENT_SUITE_SUBPARSER || suite == DEPLOYMENT_SUITE_ALL` -- YES
- Line 122-123: `suite == DEPLOYMENT_SUITE_ROUTE_PROCESSOR || (suite == DEPLOYMENT_SUITE_ALL && routeProcessor == address(0))` -- YES (conditionally)
- Line 137: `suite == DEPLOYMENT_SUITE_ARB || suite == DEPLOYMENT_SUITE_ALL` -- YES

All four deployment blocks are entered. The `all` suite deploys: OrderBookV6, OrderBookV6SubParser, RouteProcessor4 (if not pre-deployed), GenericPoolOrderBookV6ArbOrderTaker, RouteProcessorOrderBookV6ArbOrderTaker, and GenericPoolOrderBookV6FlashBorrower. This is all contracts. Verified correct.

**`BadRouteProcessor` trigger condition verification:**
- Line 129-131: `extcodehash(routeProcessor)` is computed via assembly.
- Line 133: compared against `ROUTE_PROCESSOR_4_BYTECODE_HASH`.
- Line 134: reverts with `BadRouteProcessor(ROUTE_PROCESSOR_4_BYTECODE_HASH, routeProcessor4BytecodeHash)` -- first arg is expected, second is actual. Matches the error signature at line 44. Correct.

**`ROUTE_PROCESSOR_4_BYTECODE_HASH` vs `ROUTE_PROCESSOR_4_CREATION_CODE` consistency:**
`ROUTE_PROCESSOR_4_BYTECODE_HASH` is the `extcodehash` (keccak256 of runtime bytecode), not the hash of the creation code. The runtime bytecode is produced by executing the creation code, so the hash cannot be verified statically from the hex literal alone. The only verification is at runtime (lines 129-135) or by deploying in a test and checking. No test currently does this (as noted in Pass 2). The hash value `0xeb3745a79c6ba48e8767b9c355b8e7b79f9d6edeca004e4bb91be4de515a7eeb` is a plausible keccak256 output. Without WebFetch access, I cannot cross-reference against the etherscan deployment, but the provenance comment on the creation code references verifiable sources.

## Findings

### A15-P5-1 [HIGH] Unconditional route processor bytecode check blocks `raindex`, `subparser`, and `route-processor` suites (confirms A15-1)

**Location:** Lines 90-92, 129-135

This finding confirms and extends Pass 1 finding A15-1. The `extcodehash` check at lines 129-135 runs unconditionally for all suite values. Additionally, the `vm.envAddress("DEPLOY_RAINDEX_ADDRESS")` call at line 91 also runs unconditionally when `raindex == address(0)` (which is true for all suites except `raindex` and `all`).

The combined effect is:

1. **`raindex` suite:** Line 70 fires, sets `raindex`. Line 90 check is false (skips env read). But lines 129-135 still run unconditionally. If `DEPLOY_ROUTE_PROCESSOR_4_ADDRESS` is not set, `routeProcessor` is `address(0)`, and `extcodehash(address(0))` returns the empty account hash (`0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470`), which does not equal `ROUTE_PROCESSOR_4_BYTECODE_HASH`. The script reverts with `BadRouteProcessor`. The `raindex` suite has no dependency on the route processor.

2. **`subparser` suite:** Line 70 is false, so `raindex` stays `address(0)`. Line 90 forces `vm.envAddress("DEPLOY_RAINDEX_ADDRESS")` which reverts if unset. Even if set, the unconditional bytecode check still applies.

3. **`route-processor` suite:** Line 70 is false. Line 90 forces `vm.envAddress("DEPLOY_RAINDEX_ADDRESS")` which reverts if unset. The route-processor suite deploys a route processor but has no dependency on `raindex`.

The route processor bytecode check (lines 129-135) and the `raindex` fallback read (lines 90-92) should both be gated to only execute when the relevant suite requires them:
- The bytecode check should run only when `suite == DEPLOYMENT_SUITE_ROUTE_PROCESSOR || suite == DEPLOYMENT_SUITE_ARB || suite == DEPLOYMENT_SUITE_ALL`.
- The `raindex` fallback read should run only when `suite == DEPLOYMENT_SUITE_SUBPARSER || suite == DEPLOYMENT_SUITE_ARB || suite == DEPLOYMENT_SUITE_ALL`.

### A15-P5-2 [MEDIUM] `vm.envAddress("DEPLOY_RAINDEX_ADDRESS")` reverts unconditionally for `route-processor` suite

**Location:** Lines 90-92

When running the `route-processor` suite:
- Line 70: `suite == DEPLOYMENT_SUITE_RAINDEX || suite == DEPLOYMENT_SUITE_ALL` is false. `raindex` stays `address(0)`.
- Line 90: `raindex == address(0)` is true.
- Line 91: `vm.envAddress("DEPLOY_RAINDEX_ADDRESS")` reverts if the env var is not set.

The `route-processor` suite only deploys the route processor contract (lines 122-128) and has no use for `raindex`. Yet the script forces the caller to provide `DEPLOY_RAINDEX_ADDRESS`. This is a correctness bug: the env var requirement does not match the suite's actual dependencies.

The fix is to move line 90-92 inside a condition that only executes when `raindex` is actually needed:
```solidity
if (suite == DEPLOYMENT_SUITE_SUBPARSER || suite == DEPLOYMENT_SUITE_ARB || suite == DEPLOYMENT_SUITE_ALL) {
    if (raindex == address(0)) {
        raindex = vm.envAddress("DEPLOY_RAINDEX_ADDRESS");
    }
}
```

### A15-P5-3 [LOW] No `require` / revert on `create` returning `address(0)` in `deployRouter()` (confirms A15-2)

**Location:** Lines 55-58

This confirms Pass 1 finding A15-2. The `create` opcode returns `address(0)` on failure. `deployRouter()` returns this zero address without checking. While the downstream `extcodehash` check (line 133) would catch this, the revert error would be `BadRouteProcessor` -- misleading since the actual issue is a failed deployment, not a wrong route processor.

An explicit check would provide a clearer error:
```solidity
assembly ("memory-safe") {
    routeProcessor4 := create(0, add(routeProcessor4Code, 0x20), mload(routeProcessor4Code))
}
require(routeProcessor4 != address(0), "RouteProcessor4 deployment failed");
```

### A15-P5-4 [LOW] No mechanism to reject unknown suite values

**Location:** Lines 61-175

If a caller sets `DEPLOYMENT_SUITE` to an unrecognized value (e.g., `"foo"`), the script does not revert with an explicit error. Instead, it silently skips all gated blocks and then reverts at the unconditional `extcodehash` check (line 133) with `BadRouteProcessor` -- a misleading error that does not indicate the real problem (unknown suite).

Compare with `lib/rain.interpreter/script/Deploy.sol` which uses an `else { revert UnknownDeploymentSuite(suite); }` pattern for unrecognized suite values.

Adding a final `else` clause or an explicit check at the end of the suite branching would provide a clear error message.

### A15-P5-5 [INFO] `ROUTE_PROCESSOR_4_BYTECODE_HASH` cannot be statically verified against `ROUTE_PROCESSOR_4_CREATION_CODE`

**Location:** Lines 38-42

The `ROUTE_PROCESSOR_4_BYTECODE_HASH` is the `keccak256` of the contract's runtime bytecode (what `extcodehash` returns), not the hash of the creation code. The runtime bytecode is a subset of the creation code (extracted during constructor execution) and cannot be derived without executing the constructor. Therefore, the consistency of these two constants cannot be verified by static analysis alone.

The script verifies this at runtime (lines 129-135), but there is no automated test that deploys the creation code and asserts the resulting `extcodehash` matches. This was also flagged in Pass 2 (A15-P2-2). The correctness of the hash value depends on the external provenance described in the comments (lines 30-37).

### A15-P5-6 [INFO] Constructor args in creation code encode `msg.sender` as owner via Ownable, not via constructor parameter

**Location:** Lines 36-37, 38-39

The comment on line 37 says "no owner addresses" which refers to the `address[] _privilegedUsers` constructor parameter being empty. This is accurate but could be misread as "the contract has no owner." In fact, RouteProcessor4 inherits Ownable and sets `msg.sender` as owner in the constructor. When deployed via `deployRouter()`, the owner will be the deployer address (the `Deploy` script contract during `vm.broadcast`, effectively the deployer EOA). This is expected behavior and not a bug, but the comment's phrasing is slightly ambiguous.
