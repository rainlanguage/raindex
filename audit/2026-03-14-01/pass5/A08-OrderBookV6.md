# A08 - Pass 5: Correctness / Intent Verification
## File: `src/concrete/ob/OrderBookV6.sol`

## Source File Evidence

**Contract:** `OrderBookV6` (inherits `IRaindexV6`, `IMetaV1_2`, `ReentrancyGuard`, `Multicall`, `OrderBookV6FlashLender`)

### Errors (lines 69-125)
- Line 69: `ReentrancyGuardReentrantCall()` - reentrancy guard error
- Line 73: `NotOrderOwner(address owner)` - wrong sender for order modification
- Line 76: `TokenMismatch()` - input/output tokens don't match across orders
- Line 79: `TokenSelfTrade()` - input token == output token
- Line 83: `TokenDecimalsMismatch()` - decimals mismatch
- Line 88: `MinimumIO(Float minimumIO, Float actualIO)` - minimum input not met
- Line 91: `SameOwner()` - two orders have same owner in clear
- Line 95: `UnsupportedCalculateInputs(uint256 inputs)` - calc expression wants inputs
- Line 99: `UnsupportedCalculateOutputs(uint256 outputs)` - calc expression has too few outputs
- Line 102: `NegativeInput()` - negative input against vault balances
- Line 105: `NegativeOutput()` - negative output against vault balances
- Line 109: `NegativeVaultBalance(Float vaultBalance)` - vault balance would go negative
- Line 113: `NegativeVaultBalanceChange(Float amount)` - negative amount applied
- Line 116: `NegativePull()` - negative pull attempted
- Line 119: `NegativePush()` - negative push attempted
- Line 122: `NegativeBounty()` - negative bounty calculated
- Line 125: `ClearZeroAmount()` - both clear output amounts zero

### Constants (lines 128-150)
- Line 129: `ORDER_LIVE = 1`
- Line 134: `ORDER_DEAD = 0`
- Line 137: `CALCULATE_ORDER_ENTRYPOINT = SourceIndexV2.wrap(0)`
- Line 140: `HANDLE_IO_ENTRYPOINT = SourceIndexV2.wrap(1)`
- Line 143: `CALCULATE_ORDER_MIN_OUTPUTS = 2`
- Line 145: `CALCULATE_ORDER_MAX_OUTPUTS = 2`
- Line 148: `HANDLE_IO_MIN_OUTPUTS = 0`
- Line 150: `HANDLE_IO_MAX_OUTPUTS = 0`

### Functions
- Line 219: `vaultBalance2(address, address, bytes32)` external view
- Line 226: `_vaultBalance(address, address, bytes32)` internal view
- Line 244: `orderExists(bytes32)` external view
- Line 249: `entask2(TaskV2[])` external nonReentrant
- Line 254: `deposit4(address, bytes32, Float, TaskV2[])` external nonReentrant
- Line 289: `withdraw4(address, bytes32, Float, TaskV2[])` external nonReentrant
- Line 328: `addOrder4(OrderConfigV4, TaskV2[])` external nonReentrant
- Line 378: `removeOrder3(OrderV4, TaskV2[])` external nonReentrant
- Line 401: `checkTokenSelfTrade(OrderV4, uint256, uint256)` internal pure
- Line 410: `quote2(QuoteV2)` external view
- Line 433: `takeOrders4(TakeOrdersConfigV5)` external nonReentrant
- Line 593: `clear3(...)` external nonReentrant
- Line 698: `calculateOrderIO(...)` internal view
- Line 822: `increaseVaultBalance(...)` internal
- Line 855: `decreaseVaultBalance(...)` internal
- Line 893: `recordVaultIO(...)` internal
- Line 919: `handleIO(...)` internal
- Line 971: `calculateClearStateChange(...)` internal pure
- Line 987: `calculateClearStateAlice(...)` internal pure
- Line 1008: `pullTokens(address, address, Float)` internal
- Line 1032: `pushTokens(address, address, Float)` internal
- Line 1052: `_nonZeroVaultId(address, address, bytes32)` internal pure
- Line 1058: `nonZeroVaultId` modifier

## Test Files Reviewed

1. `test/concrete/ob/OrderBookV6.addOrder.t.sol`
2. `test/concrete/ob/OrderBookV6.takeOrder.noop.t.sol`
3. `test/concrete/ob/OrderBookV6.takeOrder.tokenMismatch.t.sol`
4. `test/concrete/ob/OrderBookV6.takeOrder.precision.t.sol`
5. `test/concrete/ob/OrderBookV6.takeOrder.maximumInput.t.sol`
6. `test/concrete/ob/OrderBookV6.takeOrder.sameToken.t.sol`
7. `test/concrete/ob/OrderBookV6.clear.mock.t.sol`
8. `test/concrete/ob/OrderBookV6.clear.zeroAmount.t.sol`
9. `test/concrete/ob/OrderBookV6.clear.sameToken.t.sol`
10. `test/concrete/ob/OrderBookV6.deposit.t.sol`
11. `test/concrete/ob/OrderBookV6.withdraw.t.sol`

## Findings

### A08-1: NatSpec on `calculateOrderIO` says amounts are "always treated as 18 decimal fixed point" but uses Floats (INFO)

**Severity:** INFO

**Location:** `src/concrete/ob/OrderBookV6.sol` line 687-689

**Description:** The NatSpec comment for `calculateOrderIO` states: "Both are always treated as 18 decimal fixed point values and then rescaled according to the order's definition of each token's actual fixed point decimals." However, the implementation uses `Float` type throughout (not 18-decimal fixed point). The Float system handles decimal scaling internally. This NatSpec appears to be a leftover from an older version that used 18-decimal fixed-point integers.

**Impact:** Documentation only. No functional issue, but could confuse developers reading the code.

---

### A08-2: `UnsupportedCalculateInputs` error is declared but never thrown (INFO)

**Severity:** INFO

**Location:** `src/concrete/ob/OrderBookV6.sol` line 95

**Description:** The error `UnsupportedCalculateInputs(uint256 inputs)` is declared with NatSpec "Thrown when calculate order expression wants inputs." However, this error is never used in the contract. In `calculateOrderIO` (line 777-778), the `inputs` parameter to `eval4` is always `new StackItem[](0)` -- there is no check that the expression does not request inputs, and therefore this error is dead code. The test `testAddOrderRealCalculateInputsReverts1` deploys without error because "this is a runtime check" (per comments), but no runtime path throws this error.

**Impact:** Dead code. The error declaration is unused, possibly a vestige from an older design. If the intent is to guard against expression inputs, the check is missing.

---

### A08-3: `NegativeInput` and `NegativeOutput` errors are declared but never thrown (INFO)

**Severity:** INFO

**Location:** `src/concrete/ob/OrderBookV6.sol` lines 102, 105

**Description:** The errors `NegativeInput()` and `NegativeOutput()` are declared but never used in the contract. The only negative-value guards are `NegativeVaultBalanceChange` (in `increaseVaultBalance`/`decreaseVaultBalance`), `NegativePull` (in `pullTokens`), `NegativePush` (in `pushTokens`), and `NegativeBounty` (in `clear3`). No path in the code throws `NegativeInput` or `NegativeOutput`.

**Impact:** Dead code. These appear to be vestiges from an older design.

---

### A08-4: `TokenDecimalsMismatch` error is declared but never thrown (INFO)

**Severity:** INFO

**Location:** `src/concrete/ob/OrderBookV6.sol` line 83

**Description:** The error `TokenDecimalsMismatch()` is declared with NatSpec "Thrown when the input and output token decimals don't match, in either direction." However, this error is never used in the contract. The Float-based architecture handles decimal differences internally, making this check unnecessary.

**Impact:** Dead code.

---

### Verified Correct Behavior

1. **Error Conditions vs. Triggers:**
   - `NotOrderOwner`: Triggered at line 383 in `removeOrder3` when `msg.sender != order.owner`. Correctly guards owner-only removal. Tested in `OrderBookV6.removeOrder.owner.t.sol`.
   - `TokenMismatch`: Triggered at line 479 in `takeOrders4` when input/output tokens differ across orders, and at line 610 in `clear3` for cross-order token mismatches. Tested extensively in `tokenMismatch.t.sol`.
   - `TokenSelfTrade`: Triggered at line 405 in `checkTokenSelfTrade` when input and output tokens are the same address. Tested in `sameToken.t.sol`.
   - `SameOwner`: Triggered at line 602 in `clear3` when both orders have the same owner. Tested in `clear.mock.t.sol`.
   - `MinimumIO`: Triggered at line 561 when `actualRelevantTakerIO < config.minimumIO`. The selection of relevant IO via `IOIsInput` flag is correct.
   - `ClearZeroAmount`: Triggered at line 683 after both outputs are confirmed zero. Tested in `clear.zeroAmount.t.sol`.
   - `NegativeBounty`: Triggered at lines 655-657 when bounty would be negative (spread between orders). This is correctly guarded.
   - `NegativeVaultBalance`: Triggered in `decreaseVaultBalance` line 878-879 when subtraction would produce negative balance. Correctly prevents vault underflow.
   - `ZeroMaximumIO`: Triggered at line 466 when `maximumIO` is not positive. Tested.
   - `ZeroDepositAmount`: Triggered at line 260 when deposit amount is not positive. Tested.
   - `ZeroWithdrawTargetAmount`: Triggered at line 299-300 when withdraw target is not positive. Tested.

2. **Constants Match Definitions:**
   - `ORDER_LIVE = 1` and `ORDER_DEAD = 0`: Documented meanings match usage. `0` is the default mapping value, so all orders start dead.
   - `CALCULATE_ORDER_ENTRYPOINT = SourceIndexV2.wrap(0)`: Source index 0 for calculate order. Matches usage at line 776.
   - `HANDLE_IO_ENTRYPOINT = SourceIndexV2.wrap(1)`: Source index 1 for handle IO. Matches usage at line 945.
   - `CALCULATE_ORDER_MIN_OUTPUTS = 2`: Two outputs required (amount and ratio). Matches check at line 785 and stack reading at lines 792-793.
   - `FLASH_FEE = 0`: Zero flash loan fee. Correctly used throughout `OrderBookV6FlashLender`.

3. **Algorithm: `calculateClearStateAlice` (line 987-1003):**
   - Alice's input = alice's outputMax * alice's IORatio (line 992). Correct: the input she demands for her output.
   - Alice's output = alice's outputMax (line 994). Correct: starts at her maximum.
   - If alice's input > bob's outputMax, cap alice's input at bob's outputMax (line 998-999). Then recalculate alice's output = cappedInput / alice's IORatio (line 1002). Correct: if bob can't provide enough input to alice, alice's output is proportionally reduced.
   - The symmetry via `calculateClearStateChange` (lines 976-981) correctly flips alice/bob to compute bob's values.

4. **Algorithm: `takeOrders4` IOIsInput branching (lines 507-530):**
   - When `IOIsInput = true`: `takerInput = min(outputMax, remainingTakerIO)`, `takerOutput = IORatio * takerInput`. Correctly implements market buying where taker limits their input.
   - When `IOIsInput = false`: `orderMaxInput = IORatio * outputMax`, `takerOutput = min(orderMaxInput, remainingTakerIO)`, `takerInput = takerOutput / IORatio`. Correctly implements market selling where taker limits their output.

5. **Token Transfer Rounding:**
   - `pullTokens` (line 1017-1023): Rounds UP truncation when pulling. This is correct as the orderbook should never receive fewer tokens than the float represents.
   - `pushTokens` (line 1043): Rounds DOWN (lossy, no increment). Correct: the orderbook should never send more tokens than the float represents. This favors the protocol.

6. **Vault ID 0 Special Behavior:**
   - `_vaultBalance` for vault 0 reads `balanceOf` and `allowance` directly, returns min. Correct for "vaultless" orders.
   - `increaseVaultBalance` for vault 0 pushes tokens directly to owner. Correct.
   - `decreaseVaultBalance` for vault 0 pulls tokens directly from owner. Correct.
   - `deposit4` and `withdraw4` reject vault ID 0 via `nonZeroVaultId` modifier. Correct: direct deposits/withdrawals don't make sense for vaultless mode.

7. **Test Correctness:**
   - All test names accurately describe the behavior they exercise.
   - Fuzz tests use appropriate `bound()` and `vm.assume()` constraints.
   - The precision test suite (`OrderBookV6TakeOrderPrecisionTest`) verifies known-bad cases from older OB versions, confirming the Float system resolves them.
   - The `testTakeOrderMaximumInputMultipleOrdersMultipleOwners` test correctly models the sequential consumption of orders with independent deposits.

8. **Interface Conformance:**
   - `IRaindexV6`: All interface functions (`vaultBalance2`, `orderExists`, `entask2`, `deposit4`, `withdraw4`, `addOrder4`, `removeOrder3`, `quote2`, `takeOrders4`, `clear3`) are implemented.
   - `IMetaV1_2`: Inherited via events.
   - `IERC3156FlashLender`: Implemented via `OrderBookV6FlashLender` parent.
   - `Multicall`: Inherited from OpenZeppelin.
   - `ReentrancyGuard`: All state-modifying functions use `nonReentrant`.

9. **`clear3` Signed Context Passing (line 635-640):** Alice's order receives `bobSignedContext` and bob's order receives `aliceSignedContext`. This is intentional and consistent across all historical versions of the interface. The NatSpec "relevant to A/B" means the context that each order's expression will receive, which comes from the counterparty's perspective of the trade. The implementation is correct for the intended design.
