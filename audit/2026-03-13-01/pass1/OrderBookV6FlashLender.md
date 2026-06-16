# Pass 1: Security — OrderBookV6FlashLender.sol

**Agent:** A04
**File:** src/abstract/OrderBookV6FlashLender.sol

## Evidence of Thorough Reading

- **Contract:** `OrderBookV6FlashLender` (abstract, line 29)
- **Functions:** supportsInterface (line 33), flashLoan (line 38), flashFee (line 70), maxFlashLoan (line 78)
- **Errors:** FlashLenderCallbackFailed (line 18)
- **Constants:** FLASH_FEE = 0 (line 23)

## Findings

### A04-1 [INFO] Misleading NatSpec on `maxFlashLoan`

**Location:** Lines 74-76

The NatSpec states "If there is an active debt then loans are disabled so the max becomes 0 until after repayment." The implementation simply returns `balanceOf(address(this))` with no active-debt tracking.

### A04-2 [LOW] `flashLoan` lacks reentrancy guard, allowing nested flash loans

**Location:** Line 38

`flashLoan` makes an external call to `receiver.onFlashLoan()` without `nonReentrant`. The concrete inheritor `OrderBookV6` applies `nonReentrant` to all other state-mutating external functions but not `flashLoan`. Nested flash loans are possible during callbacks. Zero-fee model and `safeTransferFrom` repayment mitigate practical exploitation.
