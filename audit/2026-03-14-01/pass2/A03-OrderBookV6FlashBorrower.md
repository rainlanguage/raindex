# Pass 2 -- Test Coverage: A03 OrderBookV6FlashBorrower

**Source file:** `src/abstract/OrderBookV6FlashBorrower.sol` (167 lines)

## Source Summary

- Errors: `BadInitiator(address)`, `BadLender(address)`, `FlashLoanFailed`
- Inherits: IERC3156FlashBorrower, ReentrancyGuard, ERC165, OrderBookV6ArbCommon
- Constructor: passes config to OrderBookV6ArbCommon
- `supportsInterface`: reports IERC3156FlashBorrower + ERC165
- `_exchange`: internal virtual, empty no-op (hook for inheritors)
- `onFlashLoan`: external. Checks BadLender (msg.sender != deterministic OB), BadInitiator (initiator != address(this)). Decodes data, calls _exchange, calls takeOrders4. Returns ON_FLASH_LOAN_CALLBACK_SUCCESS.
- `arb4`: external payable, nonReentrant, onlyValidTask. Reverts NoOrders if empty. Reads tokens, gets decimals, computes flashLoanAmount from minimumIO. forceApprove both tokens -> OB, flashLoan, revoke both approvals, finalizeArb. Reverts FlashLoanFailed if flashLoan returns false.

## Test Files Found

| Test file | What it covers |
|-----------|---------------|
| `test/abstract/OrderBookV6FlashBorrower.ierc165.t.sol` | supportsInterface for IERC165 + IERC3156FlashBorrower, rejects bad IDs |
| `test/abstract/OrderBookV6FlashBorrower.noOrders.t.sol` | arb4 reverts NoOrders with empty array |
| `test/abstract/OrderBookV6FlashBorrower.wrongTask.t.sol` | arb4 reverts WrongTask with mismatched task |
| `test/abstract/OrderBookV6FlashBorrower.badInitiator.t.sol` | onFlashLoan reverts BadInitiator when initiator != self |
| `test/abstract/OrderBookV6FlashBorrower.lenderValidation.t.sol` | onFlashLoan reverts BadLender when called by non-OB contract |
| `test/abstract/OrderBookV6FlashBorrower.badLenderApproval.t.sol` | Malicious OB triggers BadLender; approvals don't persist |
| `test/abstract/OrderBookV6FlashBorrower.flashLoanFailed.t.sol` | arb4 reverts FlashLoanFailed when flashLoan returns false |
| `test/abstract/OrderBookV6FlashBorrower.realTokenTransfers.t.sol` | Full arb4 cycle with real ERC20 transfers |
| `test/abstract/OrderBookV6FlashBorrower.mixedDecimals.t.sol` | arb4 with 6-decimal output + 18-decimal input |
| `test/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sender.t.sol` | arb4 end-to-end with mocked OB |
| `test/concrete/arb/GenericPoolOrderBookV6FlashBorrower.ethForwarded.t.sol` | ETH forwarded during _exchange |
| `test/concrete/arb/GenericPoolOrderBookV6FlashBorrower.approvalRevoked.t.sol` | Spender approval revoked after _exchange |
| `test/concrete/arb/GenericPoolOrderBookV6FlashBorrower.exchangeRevert.t.sol` | Exchange revert propagates through arb4 |

## Coverage Analysis

### Well-covered

- supportsInterface (positive + negative fuzz)
- arb4 NoOrders revert
- arb4 WrongTask revert (fuzzed evaluable)
- onFlashLoan BadInitiator (fuzzed bad initiator)
- onFlashLoan BadLender (malicious lender contract)
- BadLender via malicious OB passed to arb4 (approval atomicity)
- FlashLoanFailed when flashLoan returns false
- Full cycle with real ERC20 transfers
- Mixed decimal tokens (6 + 18)
- _exchange hook (tested through GenericPool concrete)

### Gaps

#### GAP-A03-1: No reentrancy test for arb4

**Severity:** Low
**Details:** `arb5` on `OrderBookV6ArbOrderTaker` has a dedicated reentrancy test (`OrderBookV6ArbOrderTaker.reentrancy.t.sol`) that verifies `nonReentrant` prevents re-entry via the `takeOrders4` callback. There is no equivalent test for `arb4` on `OrderBookV6FlashBorrower`. While `arb4` also uses `nonReentrant`, explicit test coverage for the flash borrower reentrancy path is missing. The flash loan callback creates a distinct reentrancy vector (via `onFlashLoan` -> `_exchange` -> re-enter `arb4`) that should be tested independently.
