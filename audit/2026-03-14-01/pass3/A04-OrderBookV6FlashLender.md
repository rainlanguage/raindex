# Pass 3: Documentation — A04 OrderBookV6FlashLender

**File:** `src/abstract/OrderBookV6FlashLender.sol`

## Evidence of Reading

**Contract:** `OrderBookV6FlashLender` (abstract), lines 29-80
- Inherits: `IERC3156FlashLender`, `ERC165`
- Uses: `SafeERC20 for IERC20`

**Error (file-level):**
- `FlashLenderCallbackFailed(bytes32 result)` — line 18

**Constant (file-level):**
- `FLASH_FEE` = 0 — line 23

**Functions:**
1. `supportsInterface(bytes4 interfaceId)` — line 33, public view virtual override, returns bool
2. `flashLoan(IERC3156FlashBorrower receiver, address token, uint256 amount, bytes calldata data)` — line 38, external override, returns bool
3. `flashFee(address, uint256)` — line 70, external pure override, returns uint256
4. `maxFlashLoan(address token)` — line 77, external view override, returns uint256

## Findings

### A04-1 [INFO] — `flashFee` parameters are unnamed

**Location:** Line 70

`flashFee` has unnamed parameters (`address, uint256`). While the function ignores both (fee is always zero), the interface (`IERC3156FlashLender`) documents them as `token` and `amount`. The `@inheritdoc` tag pulls the interface docs, so the NatSpec is technically inherited. However, naming the parameters in the implementation improves readability for anyone reading the source directly.

**Recommendation:** Name the parameters to match the interface (`address token, uint256 amount`) even though they are unused, or add an explicit comment explaining why they are omitted. This is purely informational; the `@inheritdoc` already provides documentation.

### A04-2 [INFO] — `supportsInterface` has no standalone documentation

**Location:** Line 33

The function uses `@inheritdoc IERC165` which pulls the generic ERC165 description. It does not document that this override also advertises `IERC3156FlashLender` support. A reader must read the implementation to discover which interface IDs are supported.

**Recommendation:** Add a brief doc comment noting the additional interface advertised, e.g.:
```solidity
/// @inheritdoc IERC165
/// @dev Also returns true for `type(IERC3156FlashLender).interfaceId`.
```

### A04-3 [INFO] — All public functions have documentation via `@inheritdoc`

All four public/external functions use `@inheritdoc` to inherit NatSpec from their respective interfaces (`IERC165`, `IERC3156FlashLender`). The interface documentation in `IERC3156FlashLender` describes parameters (`token`, `amount`, `receiver`, `data`) and return values. The file-level `FlashLenderCallbackFailed` error and `FLASH_FEE` constant both have doc comments. The `maxFlashLoan` function additionally has a custom comment (line 74-75) explaining the orderbook-specific behaviour. Documentation is complete and accurate relative to the implementation.

No LOW or higher findings. No fixes needed.
