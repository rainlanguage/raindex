# raindex_math — Architecture & Design Notes

This crate provides small, focused, and overflow‑safe helpers for 256‑bit integer math used across the Rain Raindex codebase. It standardizes two things:

- Fixed‑point arithmetic in 18‑decimals (a.k.a. “wad” math).
- Safe scaling between token native decimals and 18‑decimals.

The implementation wraps `alloy::primitives` big‑ints and exposes a trait implemented for `U256` so call sites can remain expressive and chainable.

## Crate Surface

- Constants
  - `ONE18: U256` — 10^18, represented as a 256‑bit integer.
  - `FIXED_POINT_DECIMALS: u8` — 18, the standard fixed‑point scale.
- Error type
  - `MathError` — unified error for math/units/width conversions.
- Trait implemented for `U256`
  - `BigUintMath`
    - `scale_up(self, by: u8) -> Result<U256, MathError>`
    - `scale_down(self, by: u8) -> Result<U256, MathError>`
    - `scale_18(self, decimals: u8) -> Result<U256, MathError>`
    - `mul_div(self, mul: U256, div: U256) -> Result<U256, MathError>`
    - `mul_18(self, other: U256) -> Result<U256, MathError>`
    - `div_18(self, other: U256) -> Result<U256, MathError>`

There are no module subtrees; everything is in `src/lib.rs` with unit tests under `#[cfg(test)]`.

## Dependencies and Types

- `alloy::primitives::{U256, U512}` — 256/512‑bit unsigned integers.
- `alloy::primitives::ruint::{FromUintError, UintTryTo}` — fallible, width‑aware conversions and helpers.
- `alloy::primitives::utils::UnitsError` — error type for unit/decimal utilities (not currently emitted by this crate’s functions, but accounted for in `MathError`).
- `thiserror::Error` — error derivation.

Note: `once_cell` is a workspace dependency but is not used inside this crate as of this revision.

## Constants

- `ONE18`
  - Defined via limbs: `U256::from_limbs([1_000_000_000_000_000_000, 0, 0, 0])`.
  - Used as the scaling factor for 18‑decimal fixed‑point ops.
- `FIXED_POINT_DECIMALS`
  - Fixed at 18 and used by `scale_18` to decide direction of scaling.

## Error Model — `MathError`

- `Overflow` — returned when checked arithmetic fails (e.g., `checked_mul`/`checked_div` or intermediate width conversions cannot fit).
- `UnitsError(UnitsError)` — passthrough for unit/decimal parsing and conversion errors (present for ergonomic composition; not produced by functions in this file at the moment).
- `FromUintErrorU256(FromUintError<U256>)` — converting into `U256` failed (e.g., narrowing from a `U512` result that won’t fit).
- `FromUintErrorU512(FromUintError<U512>)` — converting into `U512` failed (kept for completeness when using `UintTryTo::<U512>` paths).

All public functions return `Result<U256, MathError>`. Division by zero surfaces as `Overflow` (via `checked_div` returning `None`).

## BigUintMath — Semantics and Rationale

All methods are implemented for `U256` and return a `U256` (or error). The guiding principles are:

- Deterministic integer arithmetic (no floats), suitable for on‑chain semantics and reproducible off‑chain analytics.
- Use 512‑bit intermediates when multiplying to avoid overflow before dividing.
- Keep API composable and minimal; callers can build richer flows on top.

### `scale_up(self, by: u8)`

- Computes `self * 10^by` with `checked_mul` to detect overflow.
- Intended to increase a value’s decimal precision. Example: USDC `6 → 18` uses `by = 12`.
- Errors
  - `Overflow` if `10^by` overflows `U256` or the product doesn’t fit in `U256`.

### `scale_down(self, by: u8)`

- Computes `self / 10^by` with `checked_div`.
- Truncates toward zero (integer division). No rounding.
- Errors
  - `Overflow` if divisor is zero due to an invalid exponent (practically unreachable for reasonable `by` but captured for safety).

### `scale_18(self, decimals: u8)`

- Converts `self` from `decimals` to 18 decimals.
- Logic
  - If `decimals > 18`: `scale_down(decimals - 18)`.
  - If `decimals < 18`: `scale_up(18 - decimals)`.
  - If `decimals == 18`: returns `self`.
- Behavior
  - Scaling down truncates fractional remainders; callers must account for this if rounding is required.

### `mul_div(self, mul: U256, div: U256)`

- Computes floor((`self * mul`) / `div`) using a 512‑bit widening multiply to avoid overflow during the product.
- Steps
  1. `self.widening_mul(mul)` → `U512` product.
  2. `checked_div(U512(div))` — detect divide‑by‑zero; keep the integer quotient.
  3. Attempt to `try_to::<U256>()` the result; fail if it doesn’t fit.
- Errors
  - `Overflow` on divide‑by‑zero or if intermediate doesn’t fit when narrowing.
  - `FromUintErrorU512`/`FromUintErrorU256` for width conversion failures.

### `mul_18(self, other: U256)`

- 18‑dec fixed‑point multiplication: `floor(self * other / 1e18)`.
- Implemented as `mul_div(other, ONE18)`.

### `div_18(self, other: U256)`

- 18‑dec fixed‑point division: `floor(self * 1e18 / other)`.
- Implemented as `mul_div(ONE18, other)`.

## Rounding and Truncation

- All divisions truncate toward zero (integer floor for non‑negative inputs).
- There is no built‑in rounding mode. If callers require “round half up”, they should bias the numerator before division, e.g. `mul_div(x + div/2, mul, div)` for appropriate domains.

## Limits and Edge Cases

- Exponent bounds: `10^by` must fit in `U256` to be meaningful. Practical ERC‑20 token decimals are typically ≤ 36; values well beyond ~77 will overflow `U256`.
- Division by zero: guarded and returned as `Overflow`.
- Sign: all values are unsigned (`U256`). If negative semantics are needed, the sign must be tracked out‑of‑band by the caller (as seen in other crates that pair magnitudes with boolean flags).

## Interactions in the Workspace

Downstream crates treat these helpers as the canonical way to:

- Normalize per‑token magnitudes to 18 decimals with `scale_18` for comparability.
- Perform fixed‑point rates and APY math using `mul_18`/`div_18` without risking 256‑bit overflow (thanks to widening intermediates).

Examples of usage (from `crates/subgraph`):

- Compute per‑vault APY over time windows:
  - Convert vault balances to 18‑dec with `scale_18`.
  - Compute annualization and ratios with `mul_18`/`div_18`.
- Convert between token denominations by applying a pair ratio via `mul_18` to capitals and net volumes.

## Tests — Intent and Coverage

`src/lib.rs` includes unit tests asserting the core behaviors:

- `test_big_uint_math_scale_18`
  - Scales up when `decimals < 18` and down when `decimals > 18`.
  - No‑op when already 18 decimals.
  - Shows that scaling down truncates remainders.
- `test_big_uint_math_mul_div`
  - Checks simple products and divisions.
  - Includes a case where a 256‑bit product would overflow, verifying `widening_mul` correctness and final narrowing.
- `test_big_uint_math_mul_18`
  - Exercises fixed‑point multiply, including a large value where a 256‑bit product would overflow without widening.
- `test_big_uint_math_div_18`
  - Exercises fixed‑point division under large inputs.

These tests collectively validate scaling correctness, overflow resistance, and fixed‑point semantics.

## Design Notes & Rationale

- Chosen representation: integers avoid floating‑point rounding and platform variance, matching on‑chain arithmetic.
- Widening strategy: `U512` intermediates for multiply‑then‑divide patterns are the standard approach to preserve precision without overflow before division.
- API locality: A single trait on `U256` keeps call sites terse and testable without introducing new wrapper types.
- Error aggregation: `MathError` centralizes math‑related failures for ergonomic propagation into higher‑level error types.

## Usage Patterns (Examples)

Assume `amount` is a token balance in its native decimals.

- Normalize to 18 decimals:
  ```rust
  use alloy::primitives::U256;
  use raindex_math::BigUintMath;

  let usdc_amount = U256::from(1_500_000u64); // 1.5 USDC with 6 decimals
  let wad = usdc_amount.scale_18(6)?;          // 1.5 * 1e18 as U256
  ```

- Apply a fixed‑point price (18‑decimals) to a quantity:
  ```rust
  let qty_18 = U256::from_dec_str("2000000000000000000")?; // 2.0
  let price_18 = U256::from_dec_str("333333333333333333")?; // 0.333333333333333333
  let value_18 = qty_18.mul_18(price_18)?;                   // floor(2.0 * 0.333..)
  ```

- Compute a ratio safely without overflow:
  ```rust
  let numerator = U256::from_dec_str("10_000_000_000_000_000_000")?;
  let denominator = U256::from_dec_str("2_000_000_000_000_000_000")?;
  let x = U256::from(10_000u64).mul_div(numerator, denominator)?; // 40_000
  ```

## Known Limitations / Considerations

- No rounding modes are provided; all divisions truncate. Callers must implement their own rounding if required.
- `UnitsError` is part of `MathError` for convenience but is not currently produced by functions in this file.
- Extremely large exponents in scaling (e.g., `by > ~77`) will overflow `U256` powers of ten.

## File Map

- `src/lib.rs` — All implementations and tests.
- `Cargo.toml` — Declares crate as `raindex_math`; depends on `alloy`, `thiserror` (and `once_cell` via workspace, unused here).

## Summary

`raindex_math` supplies the minimal, safe building blocks needed for consistent 18‑dec fixed‑point math on `U256`, with careful overflow handling and 512‑bit intermediates for the common mul‑then‑div pattern. Other Rain Raindex crates rely on these helpers to normalize magnitudes and compute rates without duplicating math or risking undefined overflow behavior.
