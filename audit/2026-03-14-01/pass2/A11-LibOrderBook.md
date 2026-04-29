# A11: LibOrderBook.sol -- Test Coverage Audit

## Source File
`src/lib/LibOrderBook.sol`

## Evidence of Thorough Reading

### Constants (lines 22-106)
- `CALLING_CONTEXT_COLUMNS = 4`
- `CONTEXT_COLUMNS = CALLING_CONTEXT_COLUMNS + 1` (= 5)
- `CONTEXT_COLUMNS_EXTENDED = CONTEXT_COLUMNS + 2 + 1 + 1` (= 9)
- Context column constants: `CONTEXT_CALLING_CONTEXT_COLUMN = 1`, `CONTEXT_CALCULATIONS_COLUMN = 2`, `CONTEXT_VAULT_INPUTS_COLUMN = 3`, `CONTEXT_VAULT_OUTPUTS_COLUMN = 4`, `CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN = 5`, `CONTEXT_SIGNED_CONTEXT_START_COLUMN = 6`
- Row constants for calling context (order hash, owner, counterparty), deposit context, withdraw context, calculations, vault IO, signed context

### Library Functions
- `doPost(bytes32[][] memory context, TaskV2[] memory post)` (lines 111-138):
  - Computes namespace from `msg.sender`
  - Qualifies namespace with `address(this)`
  - Iterates over `post` tasks
  - Skips tasks with empty bytecode (`task.evaluable.bytecode.length > 0` check at line 119)
  - Evaluates via `eval4` with empty inputs and empty state overlay
  - Calls `store.set(namespace, writes)` only if `writes.length > 0` (line 133)

## Test Files Found
- No direct unit test file for `LibOrderBook.doPost`
- Indirect coverage via `test/concrete/ob/OrderBookV6.deposit.entask.t.sol` (deposit4 -> doPost)
- Indirect coverage via `test/concrete/ob/OrderBookV6.withdraw.entask.t.sol` (withdraw4 -> doPost)
- Indirect coverage via `test/lib/LibOrderBookArb.finalizeArbTokenTransfers.t.sol` (finalizeArb -> doPost)
- Indirect coverage via `test/lib/LibOrderBookArb.finalizeArbNativeGas.t.sol` (finalizeArb -> doPost)

## Coverage Gaps

### GAP-A11-1: No direct unit test for `doPost` with empty `post` array
**Severity**: Low
**Details**: `doPost` is only tested indirectly through higher-level functions (deposit4, withdraw4, finalizeArb). There is no direct test that calls `doPost` with an empty `post` array (`post.length == 0`) to verify it is a no-op. The deposit/withdraw entask tests pass `new bytes[](0)` for eval strings, but those don't create tasks -- they pass an empty `TaskV2[]` directly to the orderbook which may or may not call `doPost` at all.

### GAP-A11-2: No test for `doPost` skipping tasks with empty bytecode
**Severity**: Low
**Details**: The function has an explicit `if (task.evaluable.bytecode.length > 0)` guard at line 119. There is no test that passes a task with zero-length bytecode to verify it is silently skipped. The deposit/withdraw entask tests always pass either no tasks or tasks with valid bytecode.

### GAP-A11-3: No test for `doPost` store.set path when writes are empty
**Severity**: Low
**Details**: Line 133 checks `if (writes.length > 0)` before calling `store.set`. While the stateless eval tests (e.g., `testOrderBookDepositEnactOneStateless` with `"_:1;"`) do exercise the case where writes are empty, there is no assertion verifying that `store.set` is NOT called. The test only checks read/write counts on the store address, which is an indirect verification.
