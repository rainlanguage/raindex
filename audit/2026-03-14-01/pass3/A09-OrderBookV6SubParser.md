# Pass 3: Documentation — A09 OrderBookV6SubParser

**File:** `src/concrete/parser/OrderBookV6SubParser.sol`

## Evidence of Reading

**Contract:** `OrderBookV6SubParser` (lines 69-301), inherits `BaseRainterpreterSubParser`.

### Public/External Functions and Methods
| # | Name | Visibility | Line | Has Doc Comment |
|---|------|-----------|------|-----------------|
| 1 | `describedByMetaV1()` | external pure | 73 | Yes (`@inheritdoc IDescribedByMetaV1`) |
| 2 | `subParserParseMeta()` | internal pure virtual override | 78 | Yes (`@inheritdoc BaseRainterpreterSubParser`) |
| 3 | `subParserWordParsers()` | internal pure virtual override | 83 | Yes (`@inheritdoc BaseRainterpreterSubParser`) |
| 4 | `subParserOperandHandlers()` | internal pure virtual override | 88 | Yes (`@inheritdoc BaseRainterpreterSubParser`) |
| 5 | `buildLiteralParserFunctionPointers()` | external pure | 93 | Yes (`@inheritdoc IParserToolingV1`) |
| 6 | `buildOperandHandlerFunctionPointers()` | external pure | 98 | Yes (`@inheritdoc IParserToolingV1`) |
| 7 | `buildSubParserWordParsers()` | external pure | 183 | Partial (`@dev` only, no `@inheritdoc` or `@return`) |

### State Variables
None.

### Contract-Level Documentation
- No `@title` tag.
- No `@notice` tag.

### Imports (lines 5-66)
- From `rain.interpreter`: `LibParseOperand`, `BaseRainterpreterSubParser`, `OperandV2`, `IParserToolingV1`
- From `rain.lib.typecast`: `LibConvert`
- From `rain.solmem`: `LibUint256Matrix`
- From `../../lib/LibOrderBookSubParser.sol`: `LibOrderBookSubParser`, `SUB_PARSER_WORD_PARSERS_LENGTH`, and all deposit/withdraw word constants
- From `../../lib/LibOrderBook.sol`: all `CONTEXT_*` constants
- From `../../generated/OrderBookV6SubParser.pointers.sol`: `DESCRIBED_BY_META_HASH`, `PARSE_META`, `SUB_PARSER_WORD_PARSERS`, `OPERAND_HANDLER_FUNCTION_POINTERS`
- From `rain.metadata`: `IDescribedByMetaV1`

### Errors/Events/Constants
None defined in this file (all constants are imported).

## Findings

### A09-1: Missing contract-level NatSpec (`@title` and `@notice`) (LOW)

**Location:** Line 69

`OrderBookV6SubParser` has no contract-level documentation at all. There is no `@title` or `@notice` explaining the contract's purpose as a sub-parser that makes orderbook context words (sender, order hash, vault balances, etc.) available to Rainlang expressions evaluated by the orderbook.

**Recommendation:** Add `@title` and `@notice` NatSpec before the contract declaration explaining the contract's role in the orderbook ecosystem.

### A09-2: `buildSubParserWordParsers` missing `@inheritdoc` or `@return` (LOW)

**Location:** Lines 181-183

The function has a `@dev` comment ("Builds the packed word-parser function pointer table for all orderbook sub-parser words across every context column.") but:
- It does not use `@inheritdoc ISubParserToolingV1` even though it implements that interface.
- It has no `@return` tag describing the packed bytes format.

The other two `build*` functions (lines 93, 98) correctly use `@inheritdoc IParserToolingV1`.

**Recommendation:** Replace the `@dev` with `@inheritdoc ISubParserToolingV1` (optionally keeping the `@dev` as supplementary documentation).

### A09-3: `buildOperandHandlerFunctionPointers` has no local documentation beyond `@inheritdoc` (INFO)

**Location:** Lines 98-179

This is a large function (80 lines) that constructs operand handler arrays for 9 different context columns (base, calling context, calculations, vault inputs, vault outputs, signers, signed context, deposits, withdrawals). The `@inheritdoc IParserToolingV1` delegates to the interface which provides only a generic description. No local `@dev` explains the specific orderbook context column layout or the mapping between column indices and handler arrays.

**Recommendation:** Add a `@dev` comment describing the context column layout and how each handler array maps to the orderbook's context structure.

### A09-4: `buildSubParserWordParsers` has no local documentation for context column mapping (INFO)

**Location:** Lines 183-300

Similar to `buildOperandHandlerFunctionPointers`, this function constructs parser arrays for 9 context columns but provides no documentation on the column structure or the mapping between parser functions and the words they handle.

**Recommendation:** Add `@dev` documentation describing the context column layout and word mappings.
