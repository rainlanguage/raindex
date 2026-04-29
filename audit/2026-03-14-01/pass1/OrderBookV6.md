# A08 — OrderBookV6.sol — Pass 1 (Security)

## Evidence of Thorough Reading

**File:** `src/concrete/ob/OrderBookV6.sol` (1062 lines)

**Contract:** `OrderBookV6` (inherits `IRaindexV6`, `IMetaV1_2`, `ReentrancyGuard`, `Multicall`, `OrderBookV6FlashLender`)

**Errors (lines 69-125):**
- `ReentrancyGuardReentrantCall()` — line 69
- `NotOrderOwner(address owner)` — line 73
- `TokenMismatch()` — line 76
- `TokenSelfTrade()` — line 79
- `TokenDecimalsMismatch()` — line 83
- `MinimumIO(Float minimumIO, Float actualIO)` — line 88
- `SameOwner()` — line 91
- `UnsupportedCalculateInputs(uint256 inputs)` — line 95
- `UnsupportedCalculateOutputs(uint256 outputs)` — line 99
- `NegativeInput()` — line 102
- `NegativeOutput()` — line 105
- `NegativeVaultBalance(Float vaultBalance)` — line 109
- `NegativeVaultBalanceChange(Float amount)` — line 113
- `NegativePull()` — line 116
- `NegativePush()` — line 119
- `NegativeBounty()` — line 122
- `ClearZeroAmount()` — line 125

**Constants (lines 129-150):**
- `ORDER_LIVE = 1` — line 129
- `ORDER_DEAD = 0` — line 134
- `CALCULATE_ORDER_ENTRYPOINT = SourceIndexV2.wrap(0)` — line 137
- `HANDLE_IO_ENTRYPOINT = SourceIndexV2.wrap(1)` — line 140
- `CALCULATE_ORDER_MIN_OUTPUTS = 2` — line 143
- `CALCULATE_ORDER_MAX_OUTPUTS = 2` — line 145
- `HANDLE_IO_MIN_OUTPUTS = 0` — line 148
- `HANDLE_IO_MAX_OUTPUTS = 0` — line 150

**Struct:**
- `OrderIOCalculationV4` — line 178

**State variables:**
- `sOrders` — line 208 (`mapping(bytes32 => uint256)`)
- `sVaultBalances` — line 215 (`mapping(address => mapping(address => mapping(bytes32 => Float)))`)

**Functions:**
- `vaultBalance2(address, address, bytes32)` — line 219 (external view)
- `_vaultBalance(address, address, bytes32)` — line 226 (internal view)
- `orderExists(bytes32)` — line 244 (external view)
- `entask2(TaskV2[])` — line 249 (external)
- `deposit4(address, bytes32, Float, TaskV2[])` — line 254 (external)
- `withdraw4(address, bytes32, Float, TaskV2[])` — line 289 (external)
- `addOrder4(OrderConfigV4, TaskV2[])` — line 328 (external)
- `removeOrder3(OrderV4, TaskV2[])` — line 378 (external)
- `checkTokenSelfTrade(OrderV4, uint256, uint256)` — line 403 (internal pure)
- `quote2(QuoteV2)` — line 410 (external view)
- `takeOrders4(TakeOrdersConfigV5)` — line 433 (external)
- `clear3(OrderV4, OrderV4, ClearConfigV2, SignedContextV1[], SignedContextV1[])` — line 593 (external)
- `calculateOrderIO(OrderV4, uint256, uint256, address, SignedContextV1[])` — line 698 (internal view)
- `increaseVaultBalance(address, address, bytes32, Float)` — line 822 (internal)
- `decreaseVaultBalance(address, address, bytes32, Float)` — line 855 (internal)
- `recordVaultIO(Float, Float, OrderIOCalculationV4)` — line 893 (internal)
- `handleIO(OrderIOCalculationV4)` — line 919 (internal)
- `calculateClearStateChange(OrderIOCalculationV4, OrderIOCalculationV4)` — line 971 (internal pure)
- `calculateClearStateAlice(OrderIOCalculationV4, OrderIOCalculationV4)` — line 987 (internal pure)
- `pullTokens(address, address, Float)` — line 1008 (internal)
- `pushTokens(address, address, Float)` — line 1032 (internal)
- `_nonZeroVaultId(address, address, bytes32)` — line 1052 (internal pure)
- `nonZeroVaultId` modifier — line 1058

## Findings

### A08-1 — LOW — `clear3` computes `aliceOrder.hash()` and `bobOrder.hash()` twice each

**Location:** `src/concrete/ob/OrderBookV6.sol`, lines 623-628

**Description:** In `clear3`, `aliceOrder.hash()` is called on line 623 for the liveness check, and then again implicitly inside `calculateOrderIO` on line 706 (called from line 635). Similarly for `bobOrder.hash()` on lines 627 and 638. This is a gas inefficiency rather than a correctness issue, but the duplicate hashing costs meaningful gas for a function that may be called frequently. The hash involves `abi.encode` of the full `OrderV4` struct which is non-trivial.

**Impact:** Gas waste only. No correctness or security impact — the hash is deterministic so re-computation returns the same value.

### A08-2 — INFO — Unused errors `UnsupportedCalculateInputs`, `NegativeInput`, `NegativeOutput`, `TokenDecimalsMismatch`

**Location:** `src/concrete/ob/OrderBookV6.sol`, lines 83, 95, 102, 105

**Description:** Several custom errors are declared but never used within the contract:
- `TokenDecimalsMismatch` (line 83) — no code path reverts with this error
- `UnsupportedCalculateInputs` (line 95) — the calculate order entrypoint sends no inputs (empty stack at line 778), so this error can never trigger
- `NegativeInput` (line 102) — never referenced in any revert
- `NegativeOutput` (line 105) — never referenced in any revert

These may be used by other contracts or are vestigial from a previous version.

**Impact:** No security impact. Dead code that may cause confusion during maintenance.
