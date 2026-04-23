# Triage — audit/2026-03-14-01

## Legend
- **PENDING** — not yet triaged
- **FIXED** — code changed
- **DOCUMENTED** — documentation/comments added
- **DISMISSED** — no action needed (with reason)
- **UPSTREAM** — fix belongs in a dependency/submodule

Where a finding was flagged in multiple passes, the primary finding is listed and duplicates are cross-referenced. Triaging the primary finding applies to all its duplicates.

---

## MEDIUM

| # | ID | Pass | File | Title | Status | Notes |
|---|-----|------|------|-------|--------|-------|
| 1 | A01-2 | P0 | AGENTS.md | No nix shell guidance in AGENTS.md | DISMISSED | Carried from prior triage; same as 2026-03-13-01 A01-2; process doc, not code |
| 2 | A07-1 | P2 | RouteProcessorOrderBookV6ArbOrderTaker.sol | No test for non-18-decimal tokens in onTakeOrders2 | FIXED | Added nonStandardDecimals.t.sol (6-decimal e2e) and lossyRounding.t.sol (lossy conversion round-up) |
| 3 | A08-1 | P2 | OrderBookV6.sol | No test for `MinimumIO` revert in `takeOrders4` | FIXED | Added minimumIO.t.sol with specific MinimumIO revert expectation |
| 4 | A08-2 | P2 | OrderBookV6.sol | No test for `SameOwner` revert in `clear3` | FIXED | Added clear.sameOwner.t.sol with fuzz test |
| 5 | A08-3 | P2 | OrderBookV6.sol | No test for `OrderExceedsMaxRatio` event/skip path in `takeOrders4` | FIXED | Added exceedsMaxRatio.t.sol with event expectation and zero IO assertion |
| 6 | GAP-A12-2 | P2 | LibOrderBookArb.sol | No test for output token profit sweep | FIXED | Added finalizeArbOutputTokenProfit.t.sol verifying 20e18 output token swept to sender |
| 7 | GAP-A12-3 | P2 | LibOrderBookArb.sol | No test verifying post-arb task context values | FIXED | Added finalizeArbTaskContext.t.sol with rainlang ensure expressions verifying Float-encoded context<1 0..2> |
| 8 | A14-1 | P2 | LibOrderBookDeploy.sol | RouteProcessor constants not tested in deploy tests | FIXED | Added RouteProcessor deploy/codehash/address/runtime tests to LibOrderBookDeploy.t.sol; added to prod fork tests |
| 9 | A14-2 | P2 | LibOrderBookDeploy.sol | `etchOrderBook` RouteProcessor branch not verified | FIXED | testEtchOrderBook and testEtchOrderBookIdempotent now assert RouteProcessor codehash |
| 10 | A14-1 | P4 | LibOrderBookDeploy.sol | Production `src/` library imports forge-std `Vm` | FIXED | Extracted `etchOrderBook` to test helper `LibEtchOrderBook`; removed `Vm` import from production code |

## LOW

| # | ID | Pass | File | Title | Status | Notes |
|---|-----|------|------|-------|--------|-------|
| 11 | A01-1 | P0 | CLAUDE.md | `@AGENTS.md` syntax is IDE-specific | DISMISSED | Carried from prior triage; process doc |
| 12 | A01-3 | P0 | AGENTS.md | Preflight check omits `forge fmt` | DISMISSED | Carried from prior triage; process doc |
| 13 | A01-4 | P0 | ~/.claude/CLAUDE.md | Global CLAUDE.md applies audit rules universally | DISMISSED | Carried from prior triage; process doc |
| 14 | A08-1 | P1 | OrderBookV6.sol | `clear3` computes order hashes twice each (gas) | DISMISSED | Carried from prior triage 2026-03-13-01; intentional for readability, gas cost minimal vs. external calls |
| 15 | A15-1 | P1 | Deploy.sol | String revert used for unknown deployment suite | DISMISSED | Carried from prior triage 2026-03-13-01; deploy script only |
| 16 | GAP-A03-1 | P2 | OrderBookV6FlashBorrower.sol | No reentrancy test for `arb4` (flash borrower path) | FIXED | Added OrderBookV6FlashBorrower.reentrancy.t.sol with ReentrantExchange that re-enters arb4 during _exchange pool call |
| 17 | A05-1 | P2 | GenericPoolOrderBookV6ArbOrderTaker.sol | No dedicated test for approval revocation after onTakeOrders2 | FIXED | Added approvalRevoked.t.sol asserting max allowance during call and zero after arb5 |
| 18 | A05-2 | P2 | GenericPoolOrderBookV6ArbOrderTaker.sol | No test for pool call revert propagation (order-taker path) | FIXED | Added exchangeRevert.t.sol using LibTestArb.setup abstraction; also refactored 3 flash borrower tests + order-taker approvalRevoked to use shared LibTestFlashBorrowerArb/LibTestArb setup helpers |
| 19 | A06-1 | P2 | GenericPoolOrderBookV6FlashBorrower.sol | No test for `spender != pool` in `_exchange` | FIXED | Added splitSpender.t.sol for both flash borrower and order-taker using SplitSpenderExchange mock (SpenderProxy + SplitSpenderPool); updated LibTestFlashBorrowerArb/LibTestArb with spender/pool overloads |
| 20 | A06-2 | P2 | GenericPoolOrderBookV6FlashBorrower.sol | No fuzz test over `exchangeData` decoding in `_exchange` | DISMISSED | Decoding is just `abi.decode(data, (address, address, bytes))` — fuzzing this tests the Solidity ABI codec, not application logic. The approve-call-revoke behavior and spender!=pool path are already covered by concrete tests |
| 21 | A07-2 | P2 | RouteProcessorOrderBookV6ArbOrderTaker.sol | No test for receive() and fallback() payable functions | FIXED | Added receiveAndFallback.t.sol with direct tests for both receive() (empty data) and fallback() (non-empty data) |
| 22 | A07-3 | P2 | RouteProcessorOrderBookV6ArbOrderTaker.sol | No fuzz test on onTakeOrders2 parameters directly | FIXED | Added onTakeOrders2Fuzz.t.sol with fuzzed Float inputs via try/catch to exercise toFixedDecimalLossy edge cases |
| 23 | A08-4 | P2 | OrderBookV6.sol | No test for `OrderZeroAmount` event/skip path in `takeOrders4` | FIXED | Added zeroAmount.t.sol with expression `_ _:0 1;:;` verifying OrderZeroAmount event and zero IO totals |
| 24 | A08-5 | P2 | OrderBookV6.sol | No test for `NegativeVaultBalance` revert | FIXED | Added negativeVaultBalance.t.sol via OrderBookV6SelfTest harness with external wrapper |
| 25 | A08-6 | P2 | OrderBookV6.sol | No test for `NegativeVaultBalanceChange` revert | FIXED | Added negativeVaultBalanceChange.t.sol testing both increase and decrease paths |
| 26 | A08-7 | P2 | OrderBookV6.sol | No test for `NegativePull` and `NegativePush` reverts | FIXED | Added negativePullPush.t.sol via OrderBookV6SelfTest with TOFU setup |
| 27 | A08-8 | P2 | OrderBookV6.sol | No test for `IOIsInput = false` branch with `minimumIO` check | FIXED | Added minimumIOIsOutput.t.sol with IOIsInput=false and MinimumIO revert |
| 28 | A08-12 | P2 | OrderBookV6.sol | No test for `Multicall` functionality | FIXED | Added multicall.t.sol with two deposit4 calls via multicall |
| 29 | A09-1 | P2 | OrderBookV6SubParser.sol | No dedicated sub-parser unit tests for deposit context words | DISMISSED | Carried from prior triage 2026-03-13-01 #49; trivial constant lookups |
| 30 | A09-2 | P2 | OrderBookV6SubParser.sol | No dedicated sub-parser unit tests for withdraw context words | DISMISSED | Same as #29 |
| 31 | A10-1 | P2 | LibOrder.sol | `testHashNotEqual` missing `vm.assume` guard for equal inputs | FIXED | Added vm.assume guard + testHashNotEqualMutatedOwner fuzz test |
| 32 | GAP-A11-1 | P2 | LibOrderBook.sol | No direct unit test for `doPost` with empty post array | FIXED | Added doPost.t.sol with testDoPostEmptyArray via entask2 |
| 33 | GAP-A11-2 | P2 | LibOrderBook.sol | No test for `doPost` skipping tasks with empty bytecode | FIXED | Added testDoPostEmptyBytecodeSkipped in doPost.t.sol |
| 34 | GAP-A11-3 | P2 | LibOrderBook.sol | No test for `doPost` store.set path when writes are empty | FIXED | Added testDoPostNoStoreSetWhenNoWrites with vm.record/vm.accesses |
| 35 | GAP-A12-1 | P2 | LibOrderBookArb.sol | No test for zero-balance token transfers path | FIXED | Added finalizeArbZeroBalance.t.sol with zero-profit arb |
| 36 | GAP-A12-4 | P2 | LibOrderBookArb.sol | No fuzz testing on finalizeArb | FIXED | Added finalizeArbFuzz.t.sol with bounded fuzz on all amounts |
| 37 | GAP-A13-1 | P2 | LibOrderBookSubParser.sol | No dedicated sub-parser unit tests for deposit/withdraw context words | DISMISSED | Same as #29 |
| 38 | A14-3 | P2 | LibOrderBookDeploy.sol | Prod fork tests do not verify RouteProcessor deployment | FIXED | Already present in _checkAllContracts at lines 22-26; also checks arb contracts |
| 39 | GAP-A15-1 | P2 | Deploy.sol | Deploy script has zero unit test coverage | DISMISSED | Carried from prior triage 2026-03-13-01 #43; deploy script tested via CI dry-runs |
| 40 | A02-1 | P3 | OrderBookV6ArbOrderTaker.sol | Constructor has no NatSpec documentation | FIXED | Added `@param config` NatSpec |
| 41 | A02-2 | P3 | OrderBookV6ArbOrderTaker.sol | `onTakeOrders2` parameter names are elided | DISMISSED | Naming unused params causes compiler warnings; elided intentionally |
| 42 | A03-1 | P3 | OrderBookV6FlashBorrower.sol | Constructor has no NatSpec documentation | FIXED | Added `@param config` NatSpec |
| 43 | A03-2 | P3 | OrderBookV6FlashBorrower.sol | `onFlashLoan` parameter names are partially elided | DISMISSED | Naming unused params causes compiler warnings; elided intentionally |
| 44 | A03-3 | P3 | OrderBookV6FlashBorrower.sol | `_exchange` hook docs missing explicit `@notice` tag | FIXED | Added `@dev` tag to `_exchange` hook NatSpec |
| 45 | A05-1 | P3 | GenericPoolOrderBookV6ArbOrderTaker.sol | Constructor has no NatSpec documentation | FIXED | Added `@param config` NatSpec |
| 46 | A05-2 | P3 | GenericPoolOrderBookV6ArbOrderTaker.sol | `onTakeOrders2` override lacks parameter documentation | FIXED | Added `@dev` describing `takeOrdersData` decoding |
| 47 | A06-1 | P3 | GenericPoolOrderBookV6FlashBorrower.sol | Constructor has no NatSpec documentation | FIXED | Added `@param config` NatSpec |
| 48 | A06-2 | P3 | GenericPoolOrderBookV6FlashBorrower.sol | `_exchange` override lacks parameter documentation | FIXED | Added `@dev` describing `exchangeData` decoding |
| 49 | A08-1 | P3 | OrderBookV6.sol | `recordVaultIO` doc comment truncated and inaccurate | FIXED | Rewrote NatSpec with complete param docs |
| 50 | A09-1 | P3 | OrderBookV6SubParser.sol | Missing contract-level NatSpec | FIXED | Added `@title` and `@notice` |
| 51 | A09-2 | P3 | OrderBookV6SubParser.sol | `buildSubParserWordParsers` missing `@inheritdoc` or `@return` | FIXED | Replaced ad-hoc `@dev` with `@inheritdoc IParserToolingV1` |
| 52 | A13-3 | P3 | LibOrderBookSubParser.sol | `authoringMetaV2()` return value doc inaccurate | FIXED | Changed `AuthoringMetaV2[][]` to `AuthoringMetaV2[]`; noted flattening |
| 53 | A14-1 | P3 | LibOrderBookDeploy.sol | `etchOrderBook` NatSpec omits RouteProcessor4 | FIXED | Already correct in new `LibEtchOrderBook.sol`; Dup: A14-1 P5 |
| 54 | A01-1 | P4 | OrderBookV6ArbCommon.sol | Unused `using LibEvaluable for EvaluableV4` directive | FIXED | Removed directive and unused imports |
| 55 | A01-2 | P4 | OrderBookV6ArbCommon.sol | Unused `BEFORE_ARB_SOURCE_INDEX` constant | FIXED | Removed constant and test reference; Dup: A01-1 P5 |
| 56 | A01-3 | P4 | OrderBookV6ArbCommon.sol | Unused `SignedContextV1` import | FIXED | Removed import |
| 57 | A01-4 | P4 | OrderBookV6ArbCommon.sol | Unused `IRaindexV6` import | FIXED | Removed import |
| 58 | A02-1 | P4 | OrderBookV6ArbOrderTaker.sol | Unused imports `EvaluableV4` and `SignedContextV1` | FIXED | Removed imports |
| 59 | A03-1 | P4 | OrderBookV6FlashBorrower.sol | Production code imports forge-std via `LibOrderBookDeploy` | FIXED | Fixed via #10; `LibOrderBookDeploy` no longer imports forge-std |
| 60 | A05-1 | P4 | GenericPoolOrderBookV6ArbOrderTaker.sol | Duplicated exchange pattern across GenericPool arb contracts | FIXED | Extracted to `LibGenericPoolExchange.exchange()` |
| 61 | A06-1 | P4 | GenericPoolOrderBookV6FlashBorrower.sol | Duplicated exchange pattern (dup of #60) | FIXED | Fixed via #60 |
| 62 | A07-1 | P4 | RouteProcessorOrderBookV6ArbOrderTaker.sol | Missing `sushixswap-v2` remapping in `foundry.toml` | FIXED | Added explicit remapping |
| 63 | A08-1 | P4 | OrderBookV6.sol | Stale `IOrderBookV1` reference in NatDoc comment | FIXED | Updated to `IRaindexV6`; Dup: A08-1 P5 |
| 64 | A08-2 | P4 | OrderBookV6.sol | Unused errors `TokenDecimalsMismatch`, `NegativeInput`, `NegativeOutput`, `UnsupportedCalculateInputs` | FIXED | Removed 4 unused error declarations + test import |
| 65 | A09-1 | P4 | OrderBookV6SubParser.sol | Missing `rain.lib.typecast` remapping in `foundry.toml` | FIXED | Added explicit remapping |
| 66 | A09-2 | P4 | OrderBookV6SubParser.sol | Bare `src/` import paths in test/script files | FIXED | Converted all 111 bare `src/` imports to relative paths across 72 files |
| 67 | A10-1 | P4 | LibOrder.sol | Inconsistent pragma version (`^0.8.18` vs `^0.8.19`) | FIXED | Changed to `^0.8.19` |
| 68 | A11-1 | P4 | LibOrderBook.sol | Unused imports from LibContext.sol | FIXED | Removed unused constant imports |
| 69 | A12-1 | P4 | LibOrderBookArb.sol | Unused import `IERC20Metadata` | FIXED | Removed unused import; Dup: A12-1 P5 |
| 70 | A13-1 | P4 | LibOrderBookSubParser.sol | Pragma version inconsistency across `src/lib/` | FIXED | Fixed via #67; Dup: A10-1 P4 (#67) |
| 71 | A13-3 | P4 | LibOrderBookSubParser.sol | Deposit metadata uses row+1 offsets instead of named constants | FIXED | Replaced `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_* + 1` with `DEPOSIT_WORD_*` constants |
| 72 | A03-1 | P5 | OrderBookV6FlashBorrower.sol | Test contract name uses "V5" instead of "V6" | FIXED | Renamed contract and function to V6 |
| 73 | A04-1 | P5 | OrderBookV6FlashLender.sol | `flashFee` does not revert for unsupported tokens per ERC-3156 | DISMISSED | OrderBook is token-agnostic (any ERC20 supported); zero fee is correct; maxFlashLoan returns 0 for unheld tokens preventing actual loans |
| 74 | A08-1 | P5 | OrderBookV6.sol | NatSpec says "18 decimal fixed point" but uses Floats | FIXED | Updated NatSpec to say "Float values" |
| 75 | A13-1 | P5 | LibOrderBookSubParser.sol | Test uses hardcoded depth `2` instead of `EXTERN_PARSE_META_BUILD_DEPTH` (1) | FIXED | Test now uses `EXTERN_PARSE_META_BUILD_DEPTH` constant |

## CodeRabbit (PR #2512)

| # | ID | File | Title | Status | Notes |
|---|-----|------|-------|--------|-------|
| 76 | CR-1 | LibGenericPoolExchange.sol | No zero-address guards for decoded spender/pool | DISMISSED | Zero-address spender causes forceApprove to fail; zero-address pool causes functionCallWithValue to fail; self-correcting |
| 77 | CR-2 | LibOrderBookDeployProd.t.sol | Comments say "Both contracts" but 3 are verified | FIXED | Comment already removed in current code |
| 78 | CR-3 | OrderBookV6FlashBorrower.sol | NatSpec references obsolete `onlyValidTask`; code uses `_beforeArb` | FIXED | Updated NatSpec to reference `_beforeArb` |
| 79 | CR-4 | Tests (multiple) | Repeated TaskV2 literal with deploy addresses should be shared helper | FIXED | Extracted to LibTestArb.noopTask() shared helper |
| 80 | CR-5 | Tests (multiple) | Inconsistent use of `address(0)` vs `LibInterpreterDeploy` constants | FIXED | LibTestArb.noopTask() and setupAndArb() use deploy constants throughout |
| 81 | CR-6 | OrderBookV6ExternalRealTest.sol | `iOrderbook` should use `LibOrderBookDeploy` constant directly | FIXED | Removed `iOrderbook` immutable; replaced all 312+ references across 46 test files with `LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS` and `IRaindexV6(...)` casts |
| 82 | CR-7 | Codebase-wide | Rename orderbook references to raindex | PENDING | |
| 83 | CR-8 | OrderBookV6ExternalRealTest.sol | `iInterpreter`, `iStore`, `iParserV2`, `iSubParser` should use deploy constants directly | PENDING | Same pattern as CR-6; immutables are assigned from deploy constants |
