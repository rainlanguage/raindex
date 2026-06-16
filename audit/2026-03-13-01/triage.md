# Triage — audit/2026-03-13-01

## Legend
- **PENDING** — not yet triaged
- **FIXED** — code changed
- **DOCUMENTED** — documentation/comments added
- **DISMISSED** — no action needed (with reason)
- **UPSTREAM** — fix belongs in a dependency/submodule

Where a finding was flagged in multiple passes, the primary finding is listed and duplicates are cross-referenced. Triaging the primary finding applies to all its duplicates.

---

## HIGH

| # | ID | Pass | File | Title | Status | Notes |
|---|-----|------|------|-------|--------|-------|
| 1 | A03-2 | P1 | OrderBookV6FlashBorrower.sol | Missing ERC20 approval for flash loan repayment token | FIXED | Added approve-call-revoke for ordersOutputToken; end-to-end test in missingApproval.t.sol |
| 2 | A03-P2-3 | P2 | OrderBookV6FlashBorrower.sol | Mock skips token transfers, hiding A03-2 | FIXED | RealisticFlashLendingMockOrderBook does real ERC3156 transfers |
| 3 | A05-1 | P1 | GenericPoolOrderBookV6ArbOrderTaker.sol | Unlimited approval to arbitrary spender with caller-controlled data | DOCUMENTED | By design: approve-call-revoke; contract stateless between ops; added inline comment |
| 4 | A05-P2-1 | P2 | GenericPoolOrderBookV6ArbOrderTaker.sol | `onTakeOrders2` is completely untested | FIXED | End-to-end test in onTakeOrders2.t.sol with realistic mock + exchange |
| 5 | A15-1 | P1 | Deploy.sol | Route processor bytecode hash check runs unconditionally | DISMISSED | Intentional: verifies route processor before deploying arb contracts |
| 6 | A15-P5-1 | P5 | Deploy.sol | Confirms A15-1 | DISMISSED | Dup of A15-1 |

## MEDIUM

| # | ID | Pass | File | Title | Status | Notes |
|---|-----|------|------|-------|--------|-------|
| 7 | A03-1 | P1 | OrderBookV6FlashBorrower.sol | Missing msg.sender (lender) validation in onFlashLoan | FIXED | Added iOrderBook immutable + BadLender check; test in lenderValidation.t.sol |
| 8 | A03-3 | P1 | OrderBookV6FlashBorrower.sol | Flash loan amount computed with wrong token decimals | FIXED | Changed inputDecimals → outputDecimals; test in mixedDecimals.t.sol |
| 9 | A05-2 | P1 | GenericPoolOrderBookV6ArbOrderTaker.sol | Arbitrary external call sends entire ETH balance | DOCUMENTED | By design: same as A05-1; contract stateless between ops; balance is normally 0 |
| 10 | A06-1 | P1 | GenericPoolOrderBookV6FlashBorrower.sol | Unlimited approval to arbitrary spender with no validation | DOCUMENTED | Dup of A05-1; approve-call-revoke; added inline comment |
| 11 | A03-P2-1 | P2 | OrderBookV6FlashBorrower.sol | onFlashLoan has zero direct test coverage for error paths | FIXED | BadInitiator tested in badInitiator.t.sol |
| 12 | A03-P2-2 | P2 | OrderBookV6FlashBorrower.sol | FlashLoanFailed error path is never tested | FIXED | Test in flashLoanFailed.t.sol |
| 13 | A03-P2-6 | P2 | OrderBookV6FlashBorrower.sol | WrongTask revert path has no test for flash borrower arb contracts | FIXED | Added wrongTask.t.sol |
| 14 | A05-P2-2 | P2 | GenericPoolOrderBookV6ArbOrderTaker.sol | No test for fallback() behavior | FIXED | Added fallback.t.sol for both arb types |
| 15 | A07-P2-1 | P2 | RouteProcessorOrderBookV6ArbOrderTaker.sol | onTakeOrders2 override has zero test coverage | FIXED | Test in onTakeOrders2.t.sol with MockRouteProcessor |
| 16 | A15-P5-2 | P5 | Deploy.sol | vm.envAddress reverts unconditionally for route-processor suite | DISMISSED | Incorrect: envAddress is in ARB branch (line 135), not route-processor branch |

## LOW

| # | ID | Pass | File | Title | Status | Notes |
|---|-----|------|------|-------|--------|-------|
| 17 | A02-1 | P1 | OrderBookV6ArbOrderTaker.sol | onTakeOrders2 callback has no access control | DISMISSED | By design: contract holds no value between ops; NatSpec documents this |
| 18 | A04-2 | P1 | OrderBookV6FlashLender.sol | flashLoan lacks reentrancy guard, nested flash loans | DISMISSED | By design: flash loan callbacks need to call takeOrders; tested in reentrant.t.sol |
| 19 | A05-3 | P1 | GenericPoolOrderBookV6ArbOrderTaker.sol | Non-payable fallback with misleading comment | FIXED | Comment fixed; fallback tested in fallback.t.sol |
| 20 | A06-2 | P1 | GenericPoolOrderBookV6FlashBorrower.sol | fallback accepts arbitrary calls without payable | FIXED | Dup of A05-3; comment fixed, fallback tested |
| 21 | A07-1 | P1 | RouteProcessorOrderBookV6ArbOrderTaker.sol | onTakeOrders2 is public with no access control | DISMISSED | By design: same as A02-1 |
| 22 | A08-1 | P1 | OrderBookV6.sol | flashLoan lacks nonReentrant guard | DISMISSED | By design: same as A04-2 |
| 23 | A15-2 | P1 | Deploy.sol | No explicit revert on failed create in deployRouter() | FIXED | Added require(routeProcessor4 != address(0)) |
| 24 | A01-P2-1 | P2 | OrderBookV6ArbCommon.sol | No test for WrongTask revert through FlashBorrower path | FIXED | Dup of A03-P2-6 |
| 25 | A01-P2-2 | P2 | OrderBookV6ArbCommon.sol | No direct unit test for constructor iTaskHash assignment | FIXED | Test in iTaskHash.t.sol for both non-empty and empty bytecode |
| 26 | A01-P2-3 | P2 | OrderBookV6ArbCommon.sol | No test for onlyValidTask bypass when iTaskHash == 0 | DISMISSED | Implicitly tested: sender tests deploy with empty expression (iTaskHash==0) and pass |
| 27 | A02-P2-1 | P2 | OrderBookV6ArbOrderTaker.sol | No test for arb5 reverting on zero orders | FIXED | Added noOrders.t.sol |
| 28 | A02-P2-2 | P2 | OrderBookV6ArbOrderTaker.sol | No test for reentrancy guard on arb5 | FIXED | Test in reentrancy.t.sol with ReentrantMockOrderBook |
| 29 | A02-P2-3 | P2 | OrderBookV6ArbOrderTaker.sol | onTakeOrders2 has no direct test | FIXED | Test in onTakeOrders2Direct.t.sol |
| 30 | A03-P2-4 | P2 | OrderBookV6FlashBorrower.sol | SwapFailed error declared but never used/tested | FIXED | Removed dead error; Dup: A03-P4-2 |
| 31 | A03-P2-5 | P2 | OrderBookV6FlashBorrower.sol | NoOrders revert path has no test through flash borrower | FIXED | Added FlashBorrower.noOrders.t.sol |
| 32 | A03-P2-7 | P2 | OrderBookV6FlashBorrower.sol | Flash loan amount wrong decimals not caught by tests | FIXED | Dup of A03-3 |
| 33 | A05-P2-3 | P2 | GenericPoolOrderBookV6ArbOrderTaker.sol | Constructor event emission tested only indirectly | DISMISSED | ArbTest constructor uses vm.expectEmit + emit Construct — this IS a test |
| 34 | A07-P2-2 | P2 | RouteProcessorOrderBookV6ArbOrderTaker.sol | No test for onTakeOrders2 called directly by attacker | FIXED | Test in onTakeOrders2Direct.t.sol |
| 35 | A07-P2-3 | P2 | RouteProcessorOrderBookV6ArbOrderTaker.sol | No test for constructor with invalid implementationData | FIXED | Test in invalidConstructor.t.sol |
| 36 | A12-P2-1 | P2 | LibOrderBookArb.sol | NonZeroBeforeArbStack and BadLender errors are dead code | FIXED | Removed both dead errors |
| 37 | A12-P2-2 | P2 | LibOrderBookArb.sol | No test verifies token transfers in finalizeArb | FIXED | Test in finalizeArbTokenTransfers.t.sol |
| 38 | A12-P2-3 | P2 | LibOrderBookArb.sol | No test verifies native gas sent to msg.sender | FIXED | Test in finalizeArbNativeGas.t.sol; made fallback payable + added receive() |
| 39 | A12-P2-4 | P2 | LibOrderBookArb.sol | No test exercises finalizeArb with realistic non-zero balances | FIXED | Covered by finalizeArbTokenTransfers.t.sol (20e18 profit) and finalizeArbNativeGas.t.sol (1 ETH) |
| 40 | A15-P2-1 | P2 | Deploy.sol | No unit test for deployRouter() return value on create failure | DISMISSED | deployRouter() removed; RP4 now deployed via LibRainDeploy.deployAndBroadcast |
| 41 | A15-P2-2 | P2 | Deploy.sol | ROUTE_PROCESSOR_4_BYTECODE_HASH constant never verified | DISMISSED | Inline constant removed; codehash now in generated pointers, verified by LibRouteProcessor4CreationCode test |
| 42 | A15-P2-3 | P2 | Deploy.sol | No test for BadRouteProcessor error path | DISMISSED | BadRouteProcessor error removed; deployAndBroadcast handles failures internally |
| 43 | A15-P2-4 | P2 | Deploy.sol | No test coverage for individual suite isolation | DISMISSED | Deploy.sol is a forge script; suite branches are a simple if/else chain, integration-tested by running them |
| 44 | A06-P2-GAP1 | P2 | GenericPoolOrderBookV6FlashBorrower.sol | No test for BadInitiator error path | FIXED | Added badInitiator.t.sol |
| 45 | A06-P2-GAP2 | P2 | GenericPoolOrderBookV6FlashBorrower.sol | No test for FlashLoanFailed error path | FIXED | Dup of A03-P2-2; same base class tested |
| 46 | A06-P2-GAP3 | P2 | GenericPoolOrderBookV6FlashBorrower.sol | No test for WrongTask on GenericPoolOrderBookV6FlashBorrower | FIXED | Dup of A03-P2-6 |
| 47 | A06-P2-GAP4 | P2 | GenericPoolOrderBookV6FlashBorrower.sol | No test for NoOrders revert in arb4 | FIXED | Dup of A03-P2-5 |
| 48 | A06-P2-GAP5 | P2 | GenericPoolOrderBookV6FlashBorrower.sol | No test for _exchange failure propagation | FIXED | Added exchangeRevert, approvalRevoked, and ethForwarded tests |
| 49 | A09-P2-GAP1 | P2 | OrderBookV6SubParser.sol | No tests for deposit word parsing (11 words, 0 tests) | DISMISSED | Trivial constant lookups into LibSubParse.subParserContext; compile-time index bounds checking via array assignments; testing requires parallel context fixture for minimal value |
| 50 | A01-P3-1 | P3 | OrderBookV6ArbCommon.sol | Struct @param name mismatch: tasks vs task | FIXED | Dup: A01-P5-1 |
| 51 | A01-P3-2 | P3 | OrderBookV6ArbCommon.sol | Missing documentation on contract, event, state variable, constructor, modifier | FIXED | |
| 52 | A01-P3-3 | P3 | OrderBookV6ArbCommon.sol | BEFORE_ARB_SOURCE_INDEX doc inaccurately scoped to flash loans | FIXED | |
| 53 | A02-P3-1 | P3 | OrderBookV6ArbOrderTaker.sol | Contract-level NatSpec missing | FIXED | |
| 54 | A02-P3-2 | P3 | OrderBookV6ArbOrderTaker.sol | Typo: "evaluabled" in BEFORE_ARB_SOURCE_INDEX doc | FIXED | |
| 55 | A02-P3-3 | P3 | OrderBookV6ArbOrderTaker.sol | onTakeOrders2 empty body contradicts interface MUST requirement | FIXED | Added @dev explaining default no-op |
| 56 | A03-P3-1 | P3 | OrderBookV6FlashBorrower.sol | Stale @title tag: OrderBookV5FlashBorrower | FIXED | Dup: A03-P4-3 |
| 57 | A03-P3-2 | P3 | OrderBookV6FlashBorrower.sol | Stale interface/contract references in arb4 param docs | FIXED | Dup: A03-P5-2 |
| 58 | A03-P3-3 | P3 | OrderBookV6FlashBorrower.sol | Missing @param for orderBook and task in arb4 | FIXED | |
| 59 | A04-P3-1 | P3 | OrderBookV6FlashLender.sol | Misleading NatSpec on maxFlashLoan | FIXED | Dup: A04-P5-1 |
| 60 | A05-P3-1 | P3 | GenericPoolOrderBookV6ArbOrderTaker.sol | Missing contract-level NatSpec | FIXED | |
| 61 | A06-P3-1 | P3 | GenericPoolOrderBookV6FlashBorrower.sol | Misleading fallback comment | FIXED | Dup of A05-3 |
| 62 | A07-P3-1 | P3 | RouteProcessorOrderBookV6ArbOrderTaker.sol | Missing contract-level NatSpec | FIXED | |
| 63 | A07-P3-2 | P3 | RouteProcessorOrderBookV6ArbOrderTaker.sol | Undocumented lossy conversion and silent discard of precision flag | FIXED | |
| 64 | A08-P3-1 | P3 | OrderBookV6.sol | Stale comment in recordVaultIO contradicts code execution order | FIXED | |
| 65 | A11-P3-1 | P3 | LibOrderBook.sol | Missing NatSpec on doPost | FIXED | |
| 66 | A12-P3-1 | P3 | LibOrderBookArb.sol | Missing NatSpec on finalizeArb | FIXED | |
| 67 | A13-P3-1 | P3 | LibOrderBookSubParser.sol | No NatSpec on any of the 28 SubParser functions | FIXED | |
| 68 | A15-P3-1 | P3 | Deploy.sol | @notice references deprecated "mumbai" testnet | FIXED | |
| 69 | A15-P3-2 | P3 | Deploy.sol | deployRouter() has no documentation | FIXED | |
| 70 | A15-P3-3 | P3 | Deploy.sol | run() has no documentation | FIXED | |
| 71 | A15-P3-4 | P3 | Deploy.sol | Five DEPLOYMENT_SUITE_* constants have no documentation | FIXED | |
| 72 | P3-A09-01 | P3 | OrderBookV6SubParser.sol | buildSubParserWordParsers() has no NatSpec | FIXED | |
| 73 | A01-P4-1 | P4 | OrderBookV6ArbCommon.sol | Multiple unused imports | FIXED | Removed 4 unused imports |
| 74 | A02-P4-1 | P4 | OrderBookV6ArbOrderTaker.sol | 10 unused imports | FIXED | Removed 10 unused imports |
| 75 | A02-P4-2 | P4 | OrderBookV6ArbOrderTaker.sol | Duplicate constant BEFORE_ARB_SOURCE_INDEX | FIXED | Removed duplicate, kept in ArbCommon |
| 76 | A02-P4-3 | P4 | OrderBookV6ArbOrderTaker.sol | Dead error NonZeroBeforeArbInputs | FIXED | Related: A12-P2-1 |
| 77 | A03-P4-1 | P4 | OrderBookV6FlashBorrower.sol | 7 unused imports | FIXED | Removed 8 unused imports + unused using statement |
| 78 | A03-P4-2 | P4 | OrderBookV6FlashBorrower.sol | Dead error SwapFailed | FIXED | Removed dead error |
| 79 | A03-P4-3 | P4 | OrderBookV6FlashBorrower.sol | Stale NatSpec @title OrderBookV5FlashBorrower | FIXED | Dup of A03-P3-1 |
| 80 | A06-P4-1 | P4 | GenericPoolOrderBookV6FlashBorrower.sol | Unused import IERC3156FlashLender | FIXED | Removed |
| 81 | A06-P4-2 | P4 | GenericPoolOrderBookV6FlashBorrower.sol | Unused import IERC3156FlashBorrower | FIXED | Removed |
| 82 | A07-P4-1 | P4 | RouteProcessorOrderBookV6ArbOrderTaker.sol | Unused import Address | FIXED | Removed import + unused using statement |
| 83 | A07-P4-2 | P4 | RouteProcessorOrderBookV6ArbOrderTaker.sol | No explicit remapping for sushixswap-v2 | DISMISSED | Resolved via libs=['lib'] fallback; explicit remapping unnecessary |
| 84 | A15-P4-1 | P4 | Deploy.sol | Unused import OrderBookV6 | FIXED | Removed; EvaluableV4/TaskV2/SignedContextV1 now from canonical sources |
| 85 | A15-P4-2 | P4 | Deploy.sol | Imports through concrete contract rather than canonical source | FIXED | Now imports from interface packages directly |
| 86 | A15-P4-3 | P4 | Deploy.sol | Inconsistent address() cast on raindex variable | DISMISSED | Cast required: TOFU_DECIMALS_DEPLOYMENT is a contract type, others are address |
| 87 | A15-P4-4 | P4 | Deploy.sol | Missing "memory-safe" annotation on second assembly block | FIXED | Added to both assembly blocks |
| 88 | A15-P4-5 | P4 | Deploy.sol | Repeated TaskV2/EvaluableV4 zero-value boilerplate | DISMISSED | Deploy script only; extracting a helper adds complexity for no runtime benefit |
| 89 | A01-P5-1 | P5 | OrderBookV6ArbCommon.sol | NatSpec @param tasks does not match field name task | FIXED | Dup of A01-P3-1 |
| 90 | A03-P5-1 | P5 | OrderBookV6FlashBorrower.sol | BadInitiator NatSpec inaccurately describes the check | FIXED | NatSpec now says "this contract" not "the order book" |
| 91 | A03-P5-2 | P5 | OrderBookV6FlashBorrower.sol | NatSpec for arb4 describes stale "access gate" evaluation | FIXED | Dup of A03-P3-2 |
| 92 | A04-P5-1 | P5 | OrderBookV6FlashLender.sol | maxFlashLoan NatSpec falsely claims active-debt disabling | FIXED | Dup of A04-P3-1 |
| 93 | A05-P5-01 | P5 | GenericPoolOrderBookV6ArbOrderTaker.sol | Misleading "Allow receiving gas" comment on non-payable fallback | FIXED | Dup of A05-3; comment updated |
| 94 | A06-P5-01 | P5 | GenericPoolOrderBookV6FlashBorrower.sol | Misleading "Allow receiving gas" comment on non-payable fallback | FIXED | Dup of A05-3; comment updated |
| 95 | A07-P5-01 | P5 | RouteProcessorOrderBookV6ArbOrderTaker.sol | Misleading "Allow receiving gas" comment on non-payable fallback | FIXED | Dup of A05-3; comment updated |
| 96 | A15-P5-3 | P5 | Deploy.sol | No require/revert on create returning address(0) (confirms A15-2) | FIXED | Dup of A15-2 |
| 97 | A15-P5-4 | P5 | Deploy.sol | No mechanism to reject unknown suite values | DISMISSED | Already handled: line 175 has revert("Unknown deployment suite") |

## INFO

| # | ID | Pass | File | Title | Status | Notes |
|---|-----|------|------|-------|--------|-------|
| 98 | A04-P3-2 | P3 | OrderBookV6FlashLender.sol | flashFee drops parameter names | DISMISSED | Idiomatic Solidity for unused params |
| 99 | A04-P3-3 | P3 | OrderBookV6FlashLender.sol | No @param/@return on maxFlashLoan | DISMISSED | Has @inheritdoc |
| 100 | A05-P3-3 | P3 | GenericPoolOrderBookV6ArbOrderTaker.sol | No impl-specific param docs on onTakeOrders2 | DISMISSED | Has @inheritdoc + contract notice |
| 101 | A06-P3-3 | P3 | GenericPoolOrderBookV6FlashBorrower.sol | Interface references stale V5 name | UPSTREAM | External dependency (rain.raindex.interface) |
| 102 | A07-P3-4 | P3 | RouteProcessorOrderBookV6ArbOrderTaker.sol | No NatSpec on iRouteProcessor | FIXED | |
| 103 | A08-P3-3 | P3 | OrderBookV6.sol | Contract-level NatSpec minimal (internal functions) | FIXED | Added @dev to 9 internal functions |
| 104 | A11-P3-4 | P3 | LibOrderBook.sol | No docs on deposit/withdraw/signed context constants | FIXED | Added group @dev comments |
| 105 | A13-P3-3 | P3 | LibOrderBookSubParser.sol | Missing @notice on LibOrderBookSubParser | FIXED | |
| 106 | A15-P3-7 | P3 | Deploy.sol | sDepCodeHashes state var no docs | FIXED | |
| 107 | A15-P3-8 | P3 | README.md | README missing deploy info | FIXED | Added Deployment section |
| 108 | CR-1 | CR | LibOrderBookArb.sol | sendValue called without zero-balance guard | FIXED | Added if (gasBalance > 0) guard |
| 109 | CR-2 | CR | LibOrderBookSubParser.sol | NatSpec word names don't match canonical parser keywords | DISMISSED | WORD_* constants ARE the canonical keywords; naming convention is consistent |
| 110 | CR-3 | CR | README.md | Deploy example missing ETH_RPC_URL / --rpc-url | FIXED | Added --rpc-url to example |
| 111 | CR-4 | CR | OrderBookV6SubParser.sol / LibOrderBookSubParser.sol | Repeated magic expression CONTEXT_COLUMNS + 2 + 1 + 1 should be named constant | FIXED | Extracted CONTEXT_COLUMNS_EXTENDED constant in LibOrderBook.sol |
| 112 | CR-5 | CR | OrderBookV6FlashBorrower.sol | Lender validation runs after forceApprove calls | FIXED | Added badLenderApproval test: malicious orderbook triggers BadLender, approvals atomically reverted |
| 113 | CR-6 | CR | OrderBookV6ArbOrderTaker.onTakeOrders2.t.sol | Missing post-execution balance assertions in testArb5RealTokenTransfers | FIXED | Added balance assertions to all three real-token-transfer tests |
| 114 | CR-7 | CR | OrderBookV6FlashBorrower.lenderValidation.t.sol | Unused RealisticFlashLendingMockOrderBook instance | FIXED | Removed unused import and instance |
| 115 | CR-8 | CR | OrderBookV6FlashBorrower.missingApproval.t.sol | Filename doesn't match test purpose | FIXED | Renamed to realTokenTransfers.t.sol |
| 116 | CR-9 | CR | RouteProcessorOrderBookV6ArbOrderTaker.onTakeOrders2.t.sol | Missing state verification assertions | FIXED | Added balance assertions for all participants |
