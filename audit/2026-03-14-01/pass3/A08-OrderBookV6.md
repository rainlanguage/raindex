# Pass 3: Documentation — A08 OrderBookV6

**File:** `src/concrete/ob/OrderBookV6.sol`

## Evidence of Reading

**Contract:** `OrderBookV6` (lines 191-1062), inherits `IRaindexV6`, `IMetaV1_2`, `ReentrancyGuard`, `Multicall`, `OrderBookV6FlashLender`.

### Public/External Functions and Methods
| # | Name | Visibility | Line | Has Doc Comment |
|---|------|-----------|------|-----------------|
| 1 | `vaultBalance2(address, address, bytes32)` | external view | 219 | Yes (`@inheritdoc IRaindexV6`) |
| 2 | `orderExists(bytes32)` | external view | 244 | Yes (`@inheritdoc IRaindexV6`) |
| 3 | `entask2(TaskV2[] calldata)` | external | 249 | Yes (`@inheritdoc IRaindexV6`) |
| 4 | `deposit4(address, bytes32, Float, TaskV2[] calldata)` | external | 254 | Yes (`@inheritdoc IRaindexV6`) |
| 5 | `withdraw4(address, bytes32, Float, TaskV2[] calldata)` | external | 289 | Yes (`@inheritdoc IRaindexV6`) |
| 6 | `addOrder4(OrderConfigV4 calldata, TaskV2[] calldata)` | external | 328 | Yes (`@inheritdoc IRaindexV6`) |
| 7 | `removeOrder3(OrderV4 calldata, TaskV2[] calldata)` | external | 378 | Yes (`@inheritdoc IRaindexV6`) |
| 8 | `quote2(QuoteV2 calldata)` | external view | 410 | Yes (`@inheritdoc IRaindexV6`) |
| 9 | `takeOrders4(TakeOrdersConfigV5 calldata)` | external | 433 | Yes (`@inheritdoc IRaindexV6`) |
| 10 | `clear3(OrderV4 memory, OrderV4 memory, ClearConfigV2 calldata, SignedContextV1[] memory, SignedContextV1[] memory)` | external | 593 | Yes (`@inheritdoc IRaindexV6`) |

### Internal Functions
| # | Name | Visibility | Line | Has Doc Comment |
|---|------|-----------|------|-----------------|
| 11 | `_vaultBalance(address, address, bytes32)` | internal view | 226 | Yes (`@dev`) |
| 12 | `checkTokenSelfTrade(OrderV4 memory, uint256, uint256)` | internal pure | 403 | Yes (`@dev` inline) |
| 13 | `calculateOrderIO(OrderV4 memory, uint256, uint256, address, SignedContextV1[] memory)` | internal view | 698 | Yes (full NatSpec with `@param` tags) |
| 14 | `increaseVaultBalance(address, address, bytes32, Float)` | internal | 822 | Yes (`@dev`) |
| 15 | `decreaseVaultBalance(address, address, bytes32, Float)` | internal | 855 | Yes (`@dev`) |
| 16 | `recordVaultIO(Float, Float, OrderIOCalculationV4 memory)` | internal | 893 | Yes (partial: NatSpec `@param` tags) |
| 17 | `handleIO(OrderIOCalculationV4 memory)` | internal | 919 | Yes (`@dev`) |
| 18 | `calculateClearStateChange(OrderIOCalculationV4 memory, OrderIOCalculationV4 memory)` | internal pure | 971 | Yes (full NatSpec with `@param` and `@return`) |
| 19 | `calculateClearStateAlice(OrderIOCalculationV4 memory, OrderIOCalculationV4 memory)` | internal pure | 987 | Yes (`@dev`) |
| 20 | `pullTokens(address, address, Float)` | internal | 1008 | Yes (`@dev`) |
| 21 | `pushTokens(address, address, Float)` | internal | 1032 | Yes (`@dev`) |
| 22 | `_nonZeroVaultId(address, address, bytes32)` | internal pure | 1052 | Yes (`@dev`) |

### Modifier
| # | Name | Line | Has Doc Comment |
|---|------|------|-----------------|
| 1 | `nonZeroVaultId(address, address, bytes32)` | 1058 | No |

### State Variables
| # | Name | Visibility | Line | Has Doc Comment |
|---|------|-----------|------|-----------------|
| 1 | `sOrders` | internal | 208 | Yes (lines 199-205, multi-line comment) |
| 2 | `sVaultBalances` | internal | 215 | Yes (`@dev` on line 210) |

### Contract-Level Documentation
- `@title` present (line 189): "OrderBookV6"
- Brief notice (line 190): "See `IRaindexV6` for more documentation."

### Struct
| # | Name | Line | Has Doc Comment |
|---|------|------|-----------------|
| 1 | `OrderIOCalculationV4` | 178 | Yes (full field-level NatSpec, lines 152-187) |

### Errors (lines 69-125)
| # | Name | Line | Has Doc Comment |
|---|------|------|-----------------|
| 1 | `ReentrancyGuardReentrantCall` | 69 | Yes (lines 67-68) |
| 2 | `NotOrderOwner(address)` | 73 | Yes (lines 71-72) |
| 3 | `TokenMismatch` | 76 | Yes (line 75) |
| 4 | `TokenSelfTrade` | 79 | Yes (line 78) |
| 5 | `TokenDecimalsMismatch` | 83 | Yes (lines 81-82) |
| 6 | `MinimumIO(Float, Float)` | 88 | Yes (lines 85-88) |
| 7 | `SameOwner` | 91 | Yes (line 90) |
| 8 | `UnsupportedCalculateInputs(uint256)` | 95 | Yes (lines 93-94) |
| 9 | `UnsupportedCalculateOutputs(uint256)` | 99 | Yes (lines 97-98) |
| 10 | `NegativeInput` | 102 | Yes (line 101) |
| 11 | `NegativeOutput` | 105 | Yes (line 104) |
| 12 | `NegativeVaultBalance(Float)` | 109 | Yes (lines 107-108) |
| 13 | `NegativeVaultBalanceChange(Float)` | 113 | Yes (lines 111-112) |
| 14 | `NegativePull` | 116 | Yes (line 115) |
| 15 | `NegativePush` | 119 | Yes (line 118) |
| 16 | `NegativeBounty` | 122 | Yes (line 121) |
| 17 | `ClearZeroAmount` | 125 | Yes (line 124) |

### Constants (lines 127-150)
| # | Name | Line | Has Doc Comment |
|---|------|------|-----------------|
| 1 | `ORDER_LIVE` | 129 | Yes (`@dev`) |
| 2 | `ORDER_DEAD` | 134 | Yes (`@dev`) |
| 3 | `CALCULATE_ORDER_ENTRYPOINT` | 137 | Yes (`@dev`) |
| 4 | `HANDLE_IO_ENTRYPOINT` | 140 | Yes (`@dev`) |
| 5 | `CALCULATE_ORDER_MIN_OUTPUTS` | 143 | Yes (`@dev`) |
| 6 | `CALCULATE_ORDER_MAX_OUTPUTS` | 145 | Yes (`@dev`) |
| 7 | `HANDLE_IO_MIN_OUTPUTS` | 148 | Yes (`@dev`) |
| 8 | `HANDLE_IO_MAX_OUTPUTS` | 150 | Yes (`@dev`) |

## Findings

### A08-1: `recordVaultIO` doc comment truncated -- missing `@param` continuation (LOW)

**Location:** Lines 887-893

The NatSpec for `recordVaultIO` reads:
```
/// Given an order, final input and output amounts and the IO calculation
/// verbatim from `_calculateOrderIO`, dispatch the handle IO entrypoint if
/// it exists and update the order owner's vault balances.
/// @param input The input amount.
/// @param output The output amount.
/// @param orderIOCalculation The order IO calculation produced by
```

The `@param orderIOCalculation` tag ends with "produced by" and is truncated. The sentence is incomplete -- it should say something like "produced by `calculateOrderIO`."

Additionally, the description says "dispatch the handle IO entrypoint if it exists" but `recordVaultIO` does NOT dispatch handle IO; it only records vault balance changes and emits `ContextV2`. The handle IO dispatch is done separately by `handleIO()`. This is a documentation accuracy error.

**Recommendation:** Complete the truncated `@param` tag and correct the description to accurately reflect that `recordVaultIO` only updates vault balances and emits context, not dispatching handle IO.

### A08-2: `nonZeroVaultId` modifier has no NatSpec (INFO)

**Location:** Line 1058

The modifier `nonZeroVaultId` at line 1058 has no NatSpec. Its companion function `_nonZeroVaultId` at line 1052 has a `@dev` comment, but the modifier itself is undocumented.

**Recommendation:** Add a `@dev` NatSpec tag on the modifier, or at minimum a comment indicating it delegates to `_nonZeroVaultId`.

### A08-3: `checkTokenSelfTrade` documentation is a `@dev` inline only, no `@param` tags (INFO)

**Location:** Lines 401-407

The function has a `@dev` tag explaining what it does ("Reverts with `TokenSelfTrade` if the input and output tokens are the same address.") but has no `@param` tags for its three parameters (`order`, `inputIOIndex`, `outputIOIndex`).

**Recommendation:** Add `@param` tags for completeness.

### A08-4: `calculateClearStateAlice` missing `@param` tags (INFO)

**Location:** Lines 984-1003

The function has a `@dev` description ("Calculates Alice's input and output given both order calculations. Alice's output is capped by Bob's max output, and her input is derived from her IO ratio.") but no `@param` or `@return` tags for its parameters and return values.

**Recommendation:** Add `@param` and `@return` tags.

### A08-5: `handleIO` missing `@param` tag (INFO)

**Location:** Lines 917-961

The function has a `@dev` description ("Persists interpreter state writes then evaluates the handle IO entrypoint for an order, if it has one.") but no `@param` tag for `orderIOCalculation`.

**Recommendation:** Add a `@param orderIOCalculation` tag.

### A08-6: Unused error `TokenDecimalsMismatch` (INFO)

**Location:** Line 83

The error `TokenDecimalsMismatch` is declared with a comment ("Thrown when the input and output token decimals don't match, in either direction.") but is never used in `OrderBookV6.sol`. This may be a leftover from a previous version.

**Recommendation:** Verify whether this error is used elsewhere in the codebase. If not, consider removing it.

### A08-7: Unused errors `NegativeInput` and `NegativeOutput` (INFO)

**Location:** Lines 102, 105

The errors `NegativeInput` and `NegativeOutput` are declared but never used in `OrderBookV6.sol`. The actual negative-amount checks use `NegativePull`, `NegativePush`, and `NegativeVaultBalanceChange` instead.

**Recommendation:** Verify whether these errors are used elsewhere. If not, consider removing them.

### A08-8: Unused error `UnsupportedCalculateInputs` (INFO)

**Location:** Line 95

The error `UnsupportedCalculateInputs(uint256 inputs)` is declared and documented ("Thrown when calculate order expression wants inputs") but is never thrown in `OrderBookV6.sol`. The `calculateOrderIO` function passes `new StackItem[](0)` as inputs and only checks outputs via `UnsupportedCalculateOutputs`.

**Recommendation:** Verify whether this error is used elsewhere. If not, consider removing it.

### A08-9: Unused constant `CALCULATE_ORDER_MAX_OUTPUTS` (INFO)

**Location:** Line 145

The constant `CALCULATE_ORDER_MAX_OUTPUTS` is declared and documented but never referenced in `OrderBookV6.sol`. Only `CALCULATE_ORDER_MIN_OUTPUTS` is used (line 785).

**Recommendation:** Verify if used elsewhere; if not, consider removing.
