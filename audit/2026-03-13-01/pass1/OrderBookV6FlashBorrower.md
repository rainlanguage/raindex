# Audit Pass 1: OrderBookV6FlashBorrower.sol

**Agent:** A03
**File:** `src/abstract/OrderBookV6FlashBorrower.sol`

## Evidence of Thorough Reading

### Contract Name
`OrderBookV6FlashBorrower` (abstract contract, line 62)

### Functions (with line numbers)
| Function | Line | Visibility |
|---|---|---|
| `constructor` | 66 | N/A |
| `supportsInterface` | 69 | public view virtual override |
| `_exchange` | 82 | internal virtual |
| `onFlashLoan` | 85 | external |
| `arb4` | 130 | external payable |

### Inheritance
- `IERC3156FlashBorrower`
- `ReentrancyGuard`
- `ERC165`
- `OrderBookV6ArbCommon`

### Types, Errors, and Constants
| Kind | Name | Line |
|---|---|---|
| error | `BadInitiator(address)` | 24 |
| error | `FlashLoanFailed()` | 27 |
| error | `SwapFailed()` | 30 |

### Imports (unused noted)
- `BadLender` imported at line 18 but never used in this contract.
- `LibBytecode` imported at line 10 but never used in this contract.
- `IInterpreterStoreV3` imported at line 14 but never used in this contract.
- `EvaluableV4`, `SignedContextV1` imported at line 16 but never used directly.
- `LibOrderBook` imported at line 17 but never used directly.
- `SwapFailed` error defined at line 30 but never used in this contract.

---

## Findings

### A03-1: Missing `msg.sender` (lender) validation in `onFlashLoan` [MEDIUM]

**Location:** `src/abstract/OrderBookV6FlashBorrower.sol`, line 85-107

**Description:**
The `onFlashLoan` function validates that `initiator == address(this)` (line 87) but does NOT validate that `msg.sender` is a trusted lender (orderbook). The `BadLender` error is imported (line 18) but never used, suggesting this check was intended but omitted.

Any external contract can call `onFlashLoan(address(arbContract), ...)` directly. Because `msg.sender` is not checked:

1. `_exchange(takeOrders, exchangeData)` executes with attacker-controlled data. In `GenericPoolOrderBookV6FlashBorrower._exchange`, the attacker controls `spender`, `pool`, and `encodedFunctionCall`, allowing arbitrary approvals and external calls.
2. `IRaindexV6(msg.sender).takeOrders4(takeOrders)` is called on the attacker's contract (`msg.sender`), which can return arbitrary data.

The practical impact is mitigated because the arb contract is designed to hold zero balance between uses (`finalizeArb` drains all tokens). However, if tokens are ever present in the contract (e.g., sent accidentally, or during a multi-step operation), they can be drained.

**Comparison to sibling contract:** `OrderBookV6ArbOrderTaker` does not have `onFlashLoan` and instead relies on `onTakeOrders2` which is called by the orderbook during `takeOrders4`.

**Note:** The `nonReentrant` modifier on `arb4` does NOT protect `onFlashLoan` because `onFlashLoan` is a separate external entry point. It can be called independently outside of any `arb4` invocation.

---

### A03-2: Missing ERC20 approval for flash loan repayment token [HIGH]

**Location:** `src/abstract/OrderBookV6FlashBorrower.sol`, lines 158-162

**Description:**
In `arb4`, line 158 approves `ordersInputToken` (the order's input / taker's output) to the orderbook:
```solidity
IERC20(ordersInputToken).forceApprove(address(orderBook), type(uint256).max);
```

Line 159 initiates a flash loan for `ordersOutputToken` (the order's output / taker's input):
```solidity
orderBook.flashLoan(this, ordersOutputToken, flashLoanAmount, data)
```

The flash lender (`OrderBookV6FlashLender.sol`, line 64) repays by pulling the loaned token from the borrower:
```solidity
IERC20(token).safeTransferFrom(address(receiver), address(this), amount + FLASH_FEE);
```

This `safeTransferFrom` pulls `ordersOutputToken` from the arb contract back to the orderbook. However, there is no `forceApprove` for `ordersOutputToken` to the orderbook anywhere in `arb4` or `onFlashLoan`. The only approval is for `ordersInputToken`.

**Impact:** Every call to `arb4` against the real `OrderBookV6FlashLender` will revert at flash loan repayment because the ERC20 `transferFrom` will fail due to insufficient allowance. The function is non-functional.

**Why tests don't catch this:** The test suite uses `FlashLendingMockOrderBook` whose `flashLoan` implementation (line 22-28 of `test/util/concrete/FlashLendingMockOrderBook.sol`) does not perform actual token transfers or repayment -- it simply calls `onFlashLoan` and returns `true`.

---

### A03-3: Flash loan amount computed with wrong token decimals [MEDIUM]

**Location:** `src/abstract/OrderBookV6FlashBorrower.sol`, line 153

**Description:**
```solidity
uint256 flashLoanAmount = LibDecimalFloat.toFixedDecimalLossless(takeOrders.minimumIO, inputDecimals);
```

`flashLoanAmount` is used as the amount to flash-borrow of `ordersOutputToken` (line 159). However, it is converted from the `Float` `minimumIO` using `inputDecimals` (the decimals of `ordersInputToken`), not `outputDecimals` (the decimals of `ordersOutputToken`).

When `IOIsInput` is `true`, `minimumIO` represents the minimum taker input, which is denominated in `ordersOutputToken`. Converting with `inputDecimals` instead of `outputDecimals` produces the wrong fixed-point value whenever the two tokens have different decimal precisions (e.g., USDT with 6 decimals vs DAI with 18 decimals).

When `IOIsInput` is `false`, `minimumIO` represents the minimum taker output, denominated in `ordersInputToken`. Using `inputDecimals` is correct for the Float-to-uint conversion, but the resulting amount is in `ordersInputToken` units being used to borrow `ordersOutputToken`, which is semantically incorrect.

**Impact:** The flash loan amount will be incorrect for token pairs with different decimal precisions, leading to either borrowing too much (revert if insufficient liquidity) or too little (arb may fail or be suboptimal).

---

### A03-4: Unused imports and dead error definition [INFO]

**Location:** `src/abstract/OrderBookV6FlashBorrower.sol`, lines 10, 14, 17, 18, 30

**Description:**
Several imports are unused in this contract:
- `LibBytecode` (line 10)
- `IInterpreterStoreV3` (line 14)
- `LibOrderBook` (line 17)
- `BadLender` (line 18) -- notably, this is the lender check that should be used in A03-1
- `EvaluableV4`, `SignedContextV1` (line 16)

Additionally, the `SwapFailed` error (line 30) is defined but never used in this contract or its concrete implementation `GenericPoolOrderBookV6FlashBorrower`.

These unused imports increase bytecode size slightly and reduce code clarity.

---

### A03-5: NatDoc title mismatch [INFO]

**Location:** `src/abstract/OrderBookV6FlashBorrower.sol`, line 32

**Description:**
The NatDoc title reads `@title OrderBookV5FlashBorrower` but the actual contract is `OrderBookV6FlashBorrower`. This is a copy-paste artifact from an earlier version.
