# A12: LibOrderBookArb.sol - Pass 1 (Security)

## Evidence of Thorough Reading

**File:** `src/lib/LibOrderBookArb.sol` (77 lines)

### Library
- `LibOrderBookArb` (line 14)

### Functions
- `finalizeArb(TaskV2 memory task, address ordersInputToken, uint8 inputDecimals, address ordersOutputToken, uint8 outputDecimals)` - line 20

### Imports
- `TaskV2` from IRaindexV6
- `IERC20` from OpenZeppelin
- `LibOrderBook` from ./LibOrderBook.sol
- `Address` from OpenZeppelin
- `SafeERC20` from OpenZeppelin
- `IERC20Metadata` from OpenZeppelin (unused import -- but not a security finding)
- `LibDecimalFloat`, `Float` from rain.math.float

### Using directives
- `SafeERC20 for IERC20` (line 15)

## Findings

No security findings. Analysis of each concern:

- **ERC20 transfers**: Uses `SafeERC20.safeTransfer` (lines 34, 45), correctly handling non-standard ERC20 tokens. Transfers only occur when balance > 0.
- **Native gas transfer**: Uses OpenZeppelin `Address.sendValue` (line 62), which handles the low-level call correctly with revert propagation.
- **Float conversion after transfer**: The `fromFixedDecimalLossyPacked` calls (lines 37, 49) use the `inputBalance` / `outputBalance` values captured BEFORE the `safeTransfer` calls. This means the Float values in context represent what was sent, not remaining balance. This is correct per the docstring ("amounts sent as Floats").
- **int256 cast of gasBalance** (line 67): The comment correctly explains that `gasBalance` (native ETH) cannot exceed `int256.max` because total ETH supply is far below that threshold. The `packLossless` call would revert on overflow anyway.
- **No reentrancy concern**: `msg.sender` receives funds, then `doPost` runs. If `msg.sender` is a contract, it could reenter during `sendValue`, but at that point all three transfers are complete and the function only calls `doPost` afterward. The function is `internal` so reentrancy guards are the caller's responsibility, and the state is consistent at each external call point.
- **No string reverts**: No revert statements in this file.
