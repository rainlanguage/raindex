# Pass 3: Documentation — A06 GenericPoolOrderBookV6FlashBorrower

**File:** `src/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sol`

## Evidence of Reading

**Contract:** `GenericPoolOrderBookV6FlashBorrower` (concrete), lines 26-53
- Inherits: `OrderBookV6FlashBorrower`
- Uses: `SafeERC20 for IERC20`, `Address for address`

**Functions/specials:**
1. `constructor(OrderBookV6ArbConfig memory config)` — line 30
2. `_exchange(TakeOrdersConfigV5 memory takeOrders, bytes memory exchangeData)` — line 33, internal virtual override
3. `receive()` — line 51, external payable
4. `fallback()` — line 52, external payable

## Findings

### A06-1 [LOW] — Constructor has no NatSpec documentation

**Location:** Line 30

The constructor takes an `OrderBookV6ArbConfig memory config` parameter but has no NatSpec documentation. The parent constructor (`OrderBookV6FlashBorrower` -> `OrderBookV6ArbCommon`) is documented with `@param config The arb config for this contract.` but this concrete constructor does not use `@inheritdoc` and provides no description. Users deploying this contract need to know what to pass.

**Recommendation:** Add NatSpec to the constructor:
```solidity
/// @param config The arb configuration. See `OrderBookV6ArbConfig` for details.
constructor(OrderBookV6ArbConfig memory config) OrderBookV6FlashBorrower(config) {}
```

### A06-2 [LOW] — `_exchange` override lacks parameter documentation for `exchangeData`

**Location:** Lines 33-47

The function uses `@inheritdoc OrderBookV6FlashBorrower`, which provides: "`_exchange` is responsible for converting the flash loaned assets into the assets required to fill the orders." and documents `takeOrders` and `exchangeData` as "As per `arb`." However, this concrete override decodes `exchangeData` as `(address spender, address pool, bytes encodedFunctionCall)` (line 34-35). This decoding format is documented at the contract level (lines 19-25) but not on the function itself. A reader looking only at the function's NatSpec gets no indication of the expected encoding.

The parent abstract `_exchange` in `OrderBookV6FlashBorrower` says `exchangeData` is "As per `arb`", and the `arb4` function says it is "Arbitrary bytes that will be passed to `_exchange`... For example, `GenericPoolOrderBookV6FlashBorrower` uses this data as a literal encoded external call." This is backwards — the parent references the child as an example. The child itself should document its own encoding format directly.

**Recommendation:** Add a `@dev` comment on the function documenting the expected encoding:
```solidity
/// @inheritdoc OrderBookV6FlashBorrower
/// @dev Decodes `exchangeData` as `(address spender, address pool, bytes encodedFunctionCall)`.
/// `spender` is approved for `type(uint256).max` of the borrowed token before the pool call.
/// `pool` receives the `encodedFunctionCall` via `functionCallWithValue` with any ETH balance.
/// Approval is revoked after the call.
```

### A06-3 [INFO] — Contract-level NatSpec is thorough and accurate

**Location:** Lines 15-25

The `@title`, `@notice`, and `@dev` tags provide a detailed description of the contract, including how `exchangeData` is decoded and the role of the `spender` parameter. The documentation accurately describes:
- The contract implements `OrderBookV6FlashBorrower` for external liquidity sources
- `exchangeData` is decoded into `spender`, `pool`, and `callData`
- `callData` is the literal encoded function call to the pool
- `spender` is the address approved to spend the input token (usually the pool itself)

This is accurate relative to the implementation.

### A06-4 [INFO] — `receive()` and `fallback()` have adequate inline documentation

**Location:** Lines 49-52

Both special functions have a shared comment (lines 49-50) explaining their purpose: allowing arbitrary calls and ETH transfers without reverting, with ETH swept by `finalizeArb`. This matches the identical pattern in `GenericPoolOrderBookV6ArbOrderTaker`.

### A06-5 [INFO] — `_exchange` internal visibility means docs are for developer audience only

The `_exchange` function is `internal`, so its documentation is relevant only to developers extending the contract, not to external callers. The contract-level NatSpec at lines 15-25 serves as the primary documentation for how callers should encode the `exchangeData` passed to `arb4`. This is an acceptable documentation strategy.
