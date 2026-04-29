# A07 — RouteProcessorOrderBookV6ArbOrderTaker.sol — Pass 1 (Security)

## Evidence of Thorough Reading

**File:** `src/concrete/arb/RouteProcessorOrderBookV6ArbOrderTaker.sol` (56 lines)

**Contract:** `RouteProcessorOrderBookV6ArbOrderTaker` (inherits `OrderBookV6ArbOrderTaker`)

**Functions:**
- `constructor(OrderBookV6ArbConfig memory config)` — line 21
- `onTakeOrders2(address inputToken, address outputToken, Float inputAmountSent, Float totalOutputAmount, bytes calldata takeOrdersData)` — line 27
- `receive()` — line 54
- `fallback()` — line 55

**Types/Errors/Constants:** None defined in this file.

**State variables:**
- `iRouteProcessor` (immutable, `IRouteProcessor`) — line 19

**Imports:**
- `IRouteProcessor` from sushixswap-v2 (line 5)
- `IERC20` from OpenZeppelin (line 6)
- `SafeERC20` from OpenZeppelin (line 7)
- `OrderBookV6ArbOrderTaker`, `OrderBookV6ArbConfig`, `Float` (line 9)
- `LibDecimalFloat` (line 10)
- `IERC20Metadata` (line 11)

**Using-for directives:**
- `SafeERC20 for IERC20` (line 16)

## Findings

No security findings identified.

### Analysis Notes

The approve-call-revoke pattern on lines 35/49 is correctly implemented: `forceApprove(max)` before the route processor call and `forceApprove(0)` after.

The `onTakeOrders2` function is called by OrderBook during `takeOrders4` (via the `config.data` callback mechanism). The parent `onTakeOrders2` at line 34 (`super.onTakeOrders2(...)`) is a no-op in the base class, which is correct.

The lossy conversion at line 40-41 for `inputTokenAmount` is acceptable as noted in the comment — the route processor only needs an approximate amount. For `outputTokenAmount` (lines 42-46), the `++` on lossy conversion correctly rounds up the minimum output to protect against receiving fewer tokens than needed.

The `iRouteProcessor` is immutable (set at construction), which prevents it from being changed after deployment.

The `receive()` and `fallback()` functions are intentionally open for the same reason as the flash borrower — to receive ETH during swaps, with all balances swept by `finalizeArb`.
