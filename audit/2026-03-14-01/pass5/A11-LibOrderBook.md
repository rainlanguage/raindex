# A11 - LibOrderBook.sol - Pass 5 (Correctness / Intent Verification)

**Source file:** `src/lib/LibOrderBook.sol` (139 lines)
**Related test files:**
- `test/lib/deploy/LibOrderBookDeploy.t.sol` (tests deploy helpers, not `LibOrderBook` logic)
- `test/lib/deploy/LibOrderBookDeployProd.t.sol` (tests production deployments, not `LibOrderBook` logic)
- No direct unit test for `doPost` exists; it is exercised indirectly through OrderBookV6 and LibOrderBookArb tests.

## Evidence Inventory

### Constants (file-level, lines 28-106)

| Constant | Value | Line |
|---|---|---|
| `CALLING_CONTEXT_COLUMNS` | 4 | 28 |
| `CONTEXT_COLUMNS` | 5 (= 4 + 1) | 30 |
| `CONTEXT_COLUMNS_EXTENDED` | 9 (= 5 + 2 + 1 + 1) | 34 |
| `CONTEXT_CALLING_CONTEXT_COLUMN` | 1 | 40 |
| `CONTEXT_CALLING_CONTEXT_ROWS` | 3 | 41 |
| `CONTEXT_CALLING_CONTEXT_ROW_ORDER_HASH` | 0 | 43 |
| `CONTEXT_CALLING_CONTEXT_ROW_ORDER_OWNER` | 1 | 44 |
| `CONTEXT_CALLING_CONTEXT_ROW_ORDER_COUNTERPARTY` | 2 | 45 |
| `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_TOKEN` | 0 | 49 |
| `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_ID` | 1 | 50 |
| `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_BEFORE` | 2 | 51 |
| `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_AFTER` | 3 | 52 |
| `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_TOKEN` | 0 | 56 |
| `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_ID` | 1 | 57 |
| `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_BEFORE` | 2 | 58 |
| `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_AFTER` | 3 | 59 |
| `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_TARGET_AMOUNT` | 4 | 60 |
| `CONTEXT_CALCULATIONS_COLUMN` | 2 | 64 |
| `CONTEXT_CALCULATIONS_ROWS` | 2 | 65 |
| `CONTEXT_CALCULATIONS_ROW_MAX_OUTPUT` | 0 | 67 |
| `CONTEXT_CALCULATIONS_ROW_IO_RATIO` | 1 | 68 |
| `CONTEXT_VAULT_INPUTS_COLUMN` | 3 | 74 |
| `CONTEXT_VAULT_OUTPUTS_COLUMN` | 4 | 77 |
| `CONTEXT_VAULT_IO_TOKEN` | 0 | 80 |
| `CONTEXT_VAULT_IO_TOKEN_DECIMALS` | 1 | 82 |
| `CONTEXT_VAULT_IO_VAULT_ID` | 2 | 84 |
| `CONTEXT_VAULT_IO_BALANCE_BEFORE` | 3 | 87 |
| `CONTEXT_VAULT_IO_BALANCE_DIFF` | 4 | 92 |
| `CONTEXT_VAULT_IO_ROWS` | 5 | 94 |
| `CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN` | 5 | 98 |
| `CONTEXT_SIGNED_CONTEXT_SIGNERS_ROWS` | 1 | 99 |
| `CONTEXT_SIGNED_CONTEXT_SIGNERS_ROW` | 0 | 100 |
| `CONTEXT_SIGNED_CONTEXT_START_COLUMN` | 6 | 104 |
| `CONTEXT_SIGNED_CONTEXT_START_ROWS` | 1 | 105 |
| `CONTEXT_SIGNED_CONTEXT_START_ROW` | 0 | 106 |

### Library: `LibOrderBook` (line 108)
- **Function:** `doPost(bytes32[][] memory context, TaskV2[] memory post)` (line 111) - internal

---

## Constants Verification

### Column layout
The context matrix has this column layout:
- Column 0: Base (sender, calling contract) -- from `LibContext`
- Column 1: Calling context (order hash, owner, counterparty)
- Column 2: Calculations (max output, IO ratio)
- Column 3: Vault inputs (token, decimals, vault ID, balance before, balance diff)
- Column 4: Vault outputs (same structure as inputs)
- Column 5: Signed context signers
- Column 6: Signed context data start

**`CALLING_CONTEXT_COLUMNS = 4`** (line 28): Columns 1-4 are the calling context columns. Verified: calling(1) + calculations(2) + vault_inputs(3) + vault_outputs(4) = 4 columns. Correct.

**`CONTEXT_COLUMNS = CALLING_CONTEXT_COLUMNS + 1 = 5`** (line 30): 4 calling columns + 1 base column = 5. Correct.

**`CONTEXT_COLUMNS_EXTENDED = CONTEXT_COLUMNS + 2 + 1 + 1 = 9`** (line 34): The NatSpec says "base columns plus 2 for signers and signed context start, 1 for deposit, 1 for withdraw." 5 + 2 (signers col 5, signed data col 6) + 1 (deposit col 7) + 1 (withdraw col 8) = 9. Correct. Verified against `LibOrderBookSubParser.authoringMetaV2()` which allocates `CONTEXT_COLUMNS_EXTENDED` entries and fills deposit at `CONTEXT_SIGNED_CONTEXT_START_COLUMN + 1` (= 7) and withdraw at `CONTEXT_SIGNED_CONTEXT_START_COLUMN + 2` (= 8).

### Row constants
- Vault IO rows: token(0), decimals(1), vault_id(2), balance_before(3), balance_diff(4), total = 5. `CONTEXT_VAULT_IO_ROWS = 5`. Correct.
- Calling context rows: order_hash(0), owner(1), counterparty(2), total = 3. `CONTEXT_CALLING_CONTEXT_ROWS = 3`. Correct.
- Calculations rows: max_output(0), io_ratio(1), total = 2. `CONTEXT_CALCULATIONS_ROWS = 2`. Correct.

### Deposit/Withdraw context row naming
The deposit and withdraw row constants reuse column 1 (`CONTEXT_CALLING_CONTEXT_COLUMN`) but with different row semantics depending on the operation. The deposit rows (token=0, vault_id=1, vault_before=2, vault_after=3) and withdraw rows (token=0, vault_id=1, vault_before=2, vault_after=3, target_amount=4) are used in separate `doPost` calls for deposit and withdraw operations respectively. This is correct -- the same column index is reused with context-specific meaning.

---

## NatSpec vs. Implementation

### `doPost` function (line 109-138)
- **NatSpec claim:** "Evaluates each task in `post` against the provided `context`. Tasks with empty bytecode are skipped."
- **Verified:** The function iterates over `post` (line 117), checks `task.evaluable.bytecode.length > 0` (line 119) to skip empty bytecode, calls `eval4` with the context merged with signed context (line 128), and persists writes to the store (lines 133-135). Correct.

### Namespace handling (lines 112-113, 125, 134)
- **`namespace`** (unqualified): `StateNamespace.wrap(uint256(uint160(msg.sender)))` -- derived from `msg.sender`. Used for `store.set()` on line 134.
- **`qualifiedNamespace`** (fully qualified): `LibNamespace.qualifyNamespace(namespace, address(this))` -- qualified with `address(this)`. Used for `eval4()` on line 125.
- **Verified:** `IInterpreterV4.eval4()` takes a `FullyQualifiedNamespace` in its `EvalV4` struct. `IInterpreterStoreV3.set()` takes an unqualified `StateNamespace` (the store qualifies it internally). Both usages are correct per their respective interface contracts.

### Context building (line 128)
- Uses `LibContext.build(context, task.signedContext)` which merges the caller-provided context matrix with task-specific signed context. Correct.

---

## Test Coverage

There is no direct unit test file for `LibOrderBook.doPost`. The function is tested indirectly:
- Through `OrderBookV6` operations (deposit, withdraw, addOrder, removeOrder, entask, clear) which all call `doPost`.
- Through `LibOrderBookArb.finalizeArb` tests which call `doPost` with empty-bytecode tasks.

The deploy tests (`LibOrderBookDeploy.t.sol`, `LibOrderBookDeployProd.t.sol`) test deployment constants and codehashes, not `LibOrderBook` library logic.

---

## Findings

No findings. All constants, NatSpec, and implementation logic are correct and consistent.
