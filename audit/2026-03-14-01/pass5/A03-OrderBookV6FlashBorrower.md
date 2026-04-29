# A03 - Pass 5: Correctness / Intent Verification
## File: `src/abstract/OrderBookV6FlashBorrower.sol`

## Evidence: Source Inventory

**Contract**: `OrderBookV6FlashBorrower` (abstract, lines 60-167)

| Item | Kind | Line |
|------|------|------|
| `BadInitiator` | error | 20 |
| `BadLender` | error | 25 |
| `FlashLoanFailed` | error | 28 |
| `supportsInterface` | function | 66-68 |
| `_exchange` | function (virtual) | 79 |
| `onFlashLoan` | function | 82-108 |
| `arb4` | function | 129-166 |

**Inheritance**: `IERC3156FlashBorrower`, `ReentrancyGuard`, `ERC165`, `OrderBookV6ArbCommon`

## Analysis

### Interface Conformance

1. **IERC3156FlashBorrower**: Requires `onFlashLoan(address initiator, address token, uint256 amount, uint256 fee, bytes calldata data) external returns (bytes32)`. The implementation at line 82 matches this signature. The return value is `ON_FLASH_LOAN_CALLBACK_SUCCESS` which is `keccak256("ERC3156FlashBorrower.onFlashLoan")` as specified by ERC-3156. Correct.

2. **IERC165**: `supportsInterface` reports `IERC3156FlashBorrower` and delegates to `super.supportsInterface` (covering `IERC165`). Correct.

### NatSpec vs. Implementation

1. **Error `BadInitiator` (line 18-20)**: NatSpec says "Thrown when the flash loan initiator is not this contract." The `onFlashLoan` at line 88-89 checks `initiator != address(this)` and reverts with `BadInitiator(initiator)`. Matches exactly.

2. **Error `BadLender` (line 23-25)**: NatSpec says "Thrown when onFlashLoan is called by an address other than the deterministic orderbook deployment." The `onFlashLoan` at line 84-86 checks `msg.sender != LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS` and reverts with `BadLender(msg.sender)`. Matches exactly.

3. **Error `FlashLoanFailed` (line 27-28)**: NatSpec says "Thrown when the flash loan fails somehow." The `arb4` at line 159-161 checks `!orderBook.flashLoan(...)` and reverts with `FlashLoanFailed()`. Matches.

4. **`_exchange` hook (line 70-79)**: NatSpec says it is "responsible for converting the flash loaned assets into the assets required to fill the orders." The default implementation is an empty no-op with a comment about "raising the ambient temperature of the room." The `SHOULD override` language is appropriate for an abstract contract with a virtual hook. Correct.

5. **`onFlashLoan` (line 81-108)**: NatSpec is `@inheritdoc IERC3156FlashBorrower`. The implementation:
   - Validates `msg.sender` is the deterministic orderbook (line 84-86).
   - Validates `initiator` is `address(this)` (line 88-89).
   - Decodes `data` as `(TakeOrdersConfigV5, bytes)` (line 92-93).
   - Calls `_exchange` hook (line 97).
   - Calls `takeOrders4` on the lender/orderbook (line 105).
   - Returns `ON_FLASH_LOAN_CALLBACK_SUCCESS` (line 107).
   All correct per ERC-3156 and the contract's documented intent.

6. **`arb4` (line 110-166)**: NatSpec describes the full flash loan arbitrage flow. The implementation:
   - Checks empty orders (line 136-138). Matches "Mimic what OB would do anyway."
   - Encodes data for flash loan callback (line 141).
   - Reads `ordersOutputToken` and `ordersInputToken` from first order (lines 144-145).
   - Gets decimals for both tokens (lines 147-148).
   - Computes flash loan amount from `takeOrders.minimumIO` using output decimals (line 152). The NatSpec says "We can't repay more than the minimum that the orders are going to give us and there's no reason to borrow less." This is correct: `minimumIO` is the minimum amount the taker expects to receive (in output tokens from orders), and the flash loan should borrow exactly that amount since it will be repaid from the order outputs.
   - Approves both tokens to orderbook (lines 157-158).
   - Calls `flashLoan` and checks return value (lines 159-161).
   - Revokes approvals (lines 162-163).
   - Calls `finalizeArb` (line 165).
   All matches the documented intent.

### Algorithms and Formulas

**Flash loan amount calculation (line 152)**:
```solidity
uint256 flashLoanAmount = LibDecimalFloat.toFixedDecimalLossless(takeOrders.minimumIO, outputDecimals);
```
`minimumIO` is a Float representing the minimum amount the taker will receive from the orders (in output token terms). The `toFixedDecimalLossless` converts this Float to a fixed-decimal integer using `outputDecimals`. This is correct because:
- The flash loan borrows the output token (what orders give us).
- We need to repay it, so we can't borrow more than we'll get.
- Borrowing exactly the minimum ensures we have enough to repay even if orders give us exactly the minimum.

**Approval pattern (lines 157-163)**:
Both `ordersInputToken` and `ordersOutputToken` are approved to the orderbook:
- `ordersInputToken` approval: needed because inside `onFlashLoan`, `takeOrders4` is called which will pull `ordersInputToken` from this contract (the taker sends input to the orderbook).
- `ordersOutputToken` approval: needed for flash loan repayment -- the orderbook's `flashLoan` will pull back the borrowed output tokens.
Both approvals are revoked to 0 after the flash loan completes. Correct.

### Tests vs. Claims

1. **`testOrderBookV5FlashBorrowerIERC165`** (ierc165.t.sol): Note the test contract is named `OrderBookV5FlashBorrowerIERC165Test` but tests V6 functionality. This is a minor naming inconsistency (V5 vs V6), but the test correctly verifies that `IERC165` and `IERC3156FlashBorrower` interface IDs are supported and a random bad ID is not. The test function correctly exercises the claim.

2. **`testOnFlashLoanBadInitiator`** (badInitiator.t.sol): Fuzz tests that `onFlashLoan` reverts with `BadInitiator` when called with any initiator other than the arb contract. Pranks as the deterministic orderbook address to isolate the initiator check. Correctly exercises the claim.

3. **`testMaliciousLenderCannotExploitOnFlashLoan`** (lenderValidation.t.sol): Tests that a `MaliciousLender` calling `onFlashLoan` directly reverts with `BadLender` because `msg.sender` is not the deterministic orderbook. Correctly exercises the claim.

4. **`testBadLenderRevertsWithApproval`** (badLenderApproval.t.sol): Tests that when `arb4` is called with a malicious orderbook, the entire tx reverts with `BadLender` and no approvals persist afterward. The `MaliciousOrderBook` records allowances during `flashLoan` before the `BadLender` check fires. After the revert, approvals are confirmed to be zero. Correctly exercises the claim that approvals don't persist on revert.

5. **`testFlashLoanFailed`** (flashLoanFailed.t.sol): Tests that `arb4` reverts with `FlashLoanFailed` when the orderbook's `flashLoan` returns false. Uses a `FalseFlashLoanMockOrderBook` (inherits `MockOrderBookBase` default which returns false). Correctly exercises the claim.

6. **`testArb4NoOrders`** (noOrders.t.sol): Tests that `arb4` with empty orders reverts with `NoOrders`. Correctly exercises the claim.

7. **`testArb4WrongTask`** (wrongTask.t.sol): Fuzz tests that `arb4` reverts with `WrongTask` when the provided task doesn't match the construction task. Uses `vm.assume` to ensure at least one field differs. Correctly exercises the claim.

8. **`testArb4RealTokenTransfers`** (realTokenTransfers.t.sol): Tests a full flash loan cycle with real ERC20 transfers. Verifies the orderbook receives input tokens, exchange receives output tokens, and arb contract is empty afterward. Correctly exercises the full flow.

9. **`testArb4MixedDecimals`** (mixedDecimals.t.sol): Tests with 6-decimal output (USDT-like) and 18-decimal input (DAI-like) tokens. Verifies the flash loan amount uses output decimals correctly. The test passes without assertion failures, meaning the `toFixedDecimalLossless` conversion correctly handles mixed decimals. Correctly exercises the mixed-decimal claim.

10. **`testExchangeRevertPropagates`** (exchangeRevert.t.sol): Tests that if the exchange reverts, the entire `arb4` reverts with the exchange's error message ("exchange failed"). Correctly exercises error propagation.

11. **`testApprovalRevokedAfterExchange`** (approvalRevoked.t.sol): Tests that after a successful `arb4`, the spender's allowance is revoked to zero (approve-call-revoke pattern). Also verifies the exchange saw max approval during the call. Correctly exercises the claim.

12. **`testEthForwardedToExchangeDuringExchange`** (ethForwarded.t.sol): Tests that ETH held by the arb contract is forwarded to the exchange pool via `functionCallWithValue` during `_exchange`, and that the arb contract has no remaining ETH afterward (swept by `finalizeArb`). Correctly exercises the claim.

### Error Conditions vs. Triggers

1. **`BadInitiator(address)`**: Triggered when `initiator != address(this)` in `onFlashLoan`. Name accurately describes: the initiator is bad (not this contract). Correct.

2. **`BadLender(address)`**: Triggered when `msg.sender != LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS` in `onFlashLoan`. Name accurately describes: the lender (caller) is bad (not the trusted orderbook). Correct.

3. **`FlashLoanFailed()`**: Triggered when `orderBook.flashLoan(...)` returns false. Name accurately describes: the flash loan failed. Correct.

4. **`IRaindexV6.NoOrders()`**: Triggered when `takeOrders.orders.length == 0`. Correct.

## Findings

### A03-1: Test contract name uses "V5" instead of "V6"

**Severity**: INFO

In `test/abstract/OrderBookV6FlashBorrower.ierc165.t.sol`, the test contract is named `OrderBookV5FlashBorrowerIERC165Test` and the test function is named `testOrderBookV5FlashBorrowerIERC165`, despite testing `OrderBookV6FlashBorrower` functionality. This is a cosmetic naming inconsistency that could cause confusion when reading test output.

**Location**: `test/abstract/OrderBookV6FlashBorrower.ierc165.t.sol:31,34`
