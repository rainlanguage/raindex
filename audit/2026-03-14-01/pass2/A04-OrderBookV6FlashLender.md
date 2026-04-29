# Pass 2 -- Test Coverage: A04 OrderBookV6FlashLender

**Source file:** `src/abstract/OrderBookV6FlashLender.sol` (80 lines)

## Source Summary

- Error: `FlashLenderCallbackFailed(bytes32 result)`
- Constant: `FLASH_FEE = 0`
- Inherits: IERC3156FlashLender, ERC165
- `supportsInterface`: reports IERC3156FlashLender + ERC165
- `flashLoan`: external. safeTransfer amount to receiver, calls receiver.onFlashLoan, checks result == ON_FLASH_LOAN_CALLBACK_SUCCESS (reverts FlashLenderCallbackFailed otherwise), safeTransferFrom amount+fee back. Returns true.
- `flashFee`: pure, returns FLASH_FEE (0)
- `maxFlashLoan`: returns IERC20(token).balanceOf(address(this))

## Test Files Found

| Test file | What it covers |
|-----------|---------------|
| `test/abstract/OrderBookV6FlashLender.ierc165.t.sol` | supportsInterface for IERC165 + IERC3156FlashLender, rejects bad IDs |
| `test/abstract/OrderBookV6FlashLender.fee.t.sol` | flashFee returns 0 for any token/amount (fuzzed) |
| `test/abstract/OrderBookV6FlashLender.maxFlashLoan.t.sol` | maxFlashLoan returns mocked balanceOf (fuzzed) |
| `test/abstract/OrderBookV6FlashLender.mockSuccess.t.sol` | flashLoan succeeds with mocked transfers and correct callback |
| `test/abstract/OrderBookV6FlashLender.transfers.t.sol` | Real token transfer cycle: success path (Bob returns tokens), fail path (Carol withholds tokens), and bad callback return |
| `test/abstract/OrderBookV6FlashLender.reentrant.t.sol` | Flash borrower can reenter OB (deposit, withdraw, addOrder, removeOrder, takeOrders, clear, vaultBalance, orderExists) |
| `test/abstract/OrderBookV6FlashLender.griefRecipient.t.sol` | Reverts for EOA receiver, bad callback value, and non-bytes32 return |

## Coverage Analysis

### Well-covered

- supportsInterface (positive + negative fuzz)
- flashFee always returns 0 (fuzzed)
- maxFlashLoan returns token balance (fuzzed)
- flashLoan success path (mocked + real tokens)
- FlashLenderCallbackFailed revert (bad return value)
- Transfer failure when borrower can't repay (Carol withholds)
- EOA receiver revert (no code at address)
- Non-bytes32 callback return revert
- Reentrancy from within flash loan (all major OB operations)
- Real token transfer accounting (Bob returns all tokens)

### Gaps

**No gaps found.** All functions and error paths are well-exercised with both unit tests and integration scenarios. The reentrancy test suite is particularly thorough, covering all major orderbook operations from within a flash loan callback.
