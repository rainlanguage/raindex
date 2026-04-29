# Pass 5: Correctness — LibOrderBook.sol

**Agent:** A11
**File:** `src/lib/LibOrderBook.sol` (125 lines)

## Evidence of Thorough Reading

- Read all 125 lines in full
- Verified all imports (lines 5-19): `CONTEXT_BASE_*` from `LibContext.sol`, `TaskV2`, `SourceIndexV2`, `StateNamespace`, `StackItem`, `EvalV4`, `LibNamespace`, `FullyQualifiedNamespace`, `LibContext`
- Verified all 35 file-level constants (lines 21-94)
- Verified `LibOrderBook` library with single function `doPost` (lines 96-125)
- Cross-referenced constant values against their usage in `OrderBookV6.sol` (lines 713-760), `OrderBookV6SubParser.sol`, and `LibOrderBookSubParser.sol`

## Correctness Verification

### File-Level Constants

#### Context Column Layout

| Constant | Value | Verified Against |
|---|---|---|
| `CALLING_CONTEXT_COLUMNS` | 4 | `OrderBookV6.calculateOrderIO` allocates `new bytes32[][](4)` |
| `CONTEXT_COLUMNS` | `CALLING_CONTEXT_COLUMNS + 1` = 5 | Base column (0) + 4 calling columns |
| `CONTEXT_CALLING_CONTEXT_COLUMN` | 1 | Used as `callingContext[1 - 1]` in OBV6 |
| `CONTEXT_CALCULATIONS_COLUMN` | 2 | Filled in `_recordVaultIO` |
| `CONTEXT_VAULT_INPUTS_COLUMN` | 3 | `callingContext[3 - 1]` in OBV6 |
| `CONTEXT_VAULT_OUTPUTS_COLUMN` | 4 | `callingContext[4 - 1]` in OBV6 |
| `CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN` | 5 | Appended by `LibContext.build` |
| `CONTEXT_SIGNED_CONTEXT_START_COLUMN` | 6 | Appended by `LibContext.build` |

All column indices are sequentially correct and consistent with the `LibContext.build` prepend behavior (base at 0, caller-provided at 1..N, signed context appended after).

#### Calling Context Rows

| Constant | Value | Verified |
|---|---|---|
| `CONTEXT_CALLING_CONTEXT_ROWS` | 3 | `arrayFrom(orderHash, owner, counterparty)` = 3 elements |
| `CONTEXT_CALLING_CONTEXT_ROW_ORDER_HASH` | 0 | First arg to `arrayFrom` |
| `CONTEXT_CALLING_CONTEXT_ROW_ORDER_OWNER` | 1 | Second arg |
| `CONTEXT_CALLING_CONTEXT_ROW_ORDER_COUNTERPARTY` | 2 | Third arg |

#### Deposit Context Rows (lines 43-46)

| Constant | Value | Actual Position in `deposit4` Context |
|---|---|---|
| `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_TOKEN` | 0 | `arrayFrom` arg 1: `token` |
| `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_ID` | 1 | `arrayFrom` arg 2: `vaultId` |
| `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_BEFORE` | 2 | `arrayFrom` arg 3: `beforeBalance` |
| `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_AFTER` | 3 | `arrayFrom` arg 4: `afterBalance` |

Verified against `OrderBookV6.deposit4` lines 279-286. Note: deposit context also has a 5th element (decimals at row 4) that has no named constant. This is intentional -- decimals are not exposed to sub-parser expressions.

#### Withdraw Context Rows (lines 48-52)

| Constant | Value | Actual Position in `withdraw4` Context |
|---|---|---|
| `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_TOKEN` | 0 | `arrayFrom` arg 1: `token` |
| `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_ID` | 1 | `arrayFrom` arg 2: `vaultId` |
| `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_BEFORE` | 2 | `arrayFrom` arg 3: `beforeBalance` |
| `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_AFTER` | 3 | `arrayFrom` arg 4: `afterBalance` |
| `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_TARGET_AMOUNT` | 4 | `arrayFrom` arg 5: `targetAmount` |

Verified against `OrderBookV6.withdraw4` lines 316-323. Withdraw context also has a 6th element (decimals at row 5) with no named constant, same as deposit.

#### Calculations Rows

| Constant | Value | Verified |
|---|---|---|
| `CONTEXT_CALCULATIONS_ROWS` | 2 | max-output and io-ratio |
| `CONTEXT_CALCULATIONS_ROW_MAX_OUTPUT` | 0 | Stack position `0x40` in assembly |
| `CONTEXT_CALCULATIONS_ROW_IO_RATIO` | 1 | Stack position `0x20` in assembly |

#### Vault IO Rows

| Constant | Value | Verified Against |
|---|---|---|
| `CONTEXT_VAULT_IO_TOKEN` | 0 | `arrayFrom` arg 1 in OBV6 |
| `CONTEXT_VAULT_IO_TOKEN_DECIMALS` | 1 | `arrayFrom` arg 2 |
| `CONTEXT_VAULT_IO_VAULT_ID` | 2 | `arrayFrom` arg 3 |
| `CONTEXT_VAULT_IO_BALANCE_BEFORE` | 3 | `arrayFrom` arg 4 |
| `CONTEXT_VAULT_IO_BALANCE_DIFF` | 4 | `arrayFrom` arg 5 (initially 0, filled later) |
| `CONTEXT_VAULT_IO_ROWS` | 5 | 5 elements in the array |

#### Signed Context

| Constant | Value | Purpose |
|---|---|---|
| `CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN` | 5 | Column for signer addresses |
| `CONTEXT_SIGNED_CONTEXT_SIGNERS_ROWS` | 1 | Single row type (indexed by operand) |
| `CONTEXT_SIGNED_CONTEXT_SIGNERS_ROW` | 0 | The single signer row |
| `CONTEXT_SIGNED_CONTEXT_START_COLUMN` | 6 | First column of signed data |
| `CONTEXT_SIGNED_CONTEXT_START_ROWS` | 1 | Single row type (indexed by operand) |
| `CONTEXT_SIGNED_CONTEXT_START_ROW` | 0 | The single signed context row |

### NatSpec on Constants

- Line 23: "calculuate" is a typo for "calculate" -- cosmetic only.
- Lines 80-83: NatSpec on `CONTEXT_VAULT_IO_BALANCE_DIFF` says "subtraced" -- should be "subtracted". Cosmetic typo.
- The `@dev` comments accurately describe the semantics: balance diff is always positive, must be added to input balances and subtracted from output balances.

### `doPost` Function (lines 97-124)

**What it claims to do:** No NatSpec (undocumented internal function).

**What it does:**
1. Wraps `msg.sender` into a `StateNamespace` (line 98)
2. Qualifies the namespace with `address(this)` (line 99)
3. Iterates over `post` tasks (line 103)
4. Skips tasks with empty bytecode (line 105)
5. Evaluates each task via `eval4` with:
   - The provided context merged with the task's signed context via `LibContext.build` (line 113)
   - Source index 0 (line 112)
   - No inputs (line 114)
   - No state overlay (line 115)
6. Writes KV pairs to the store if any (lines 119-121)
7. Discards the stack (line 118)

**Caller expectations verified:**
- `OrderBookV6.deposit4/withdraw4/addOrder4/removeOrder3/entask2`: All pass appropriate context and post tasks. The namespace = `msg.sender` means the expression state is scoped to the caller (depositor/withdrawer/order owner).
- `LibOrderBookArb.finalizeArb`: Passes arb context (input/output/gas balances). When called from arb contracts, `msg.sender` is the arb initiator, which is correct for scoping arb task state to the person who triggered the arb.

## Findings

### P5-A11-01 (INFO): Typos in NatSpec Comments

**Severity:** INFO
**File:** `src/lib/LibOrderBook.sol`, lines 23 and 83

**Details:**
- Line 23: "calculuate" should be "calculate"
- Line 83: "subtraced" should be "subtracted"

**Impact:** Documentation-only. No functional impact.

## Summary

| ID | Severity | Description |
|---|---|---|
| P5-A11-01 | INFO | Typos in NatSpec: "calculuate" and "subtraced" |

All 35 constants have been verified against their usage sites in the orderbook, sub-parser, and arb contracts. Every column index, row index, and row count is consistent with the actual context arrays constructed at runtime. The `doPost` function correctly evaluates post tasks in the caller's namespace.
