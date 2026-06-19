/**
 * The maximum number of fractional (post-decimal-point) digits that the
 * underlying `Float` decimal parser accepts without precision loss. A value
 * with more than this many decimal places is rejected client-side with a
 * friendly message.
 */
export const MAX_DECIMALS = 67;

export interface AmountDecimalsValidation {
  isValid: boolean;
  errorMessage: string | null;
}

/**
 * Counts the number of fractional digits in a plain decimal string.
 *
 * Returns `null` when the input is not a plain decimal number (empty, or
 * containing anything other than an optional leading sign and digits around a
 * single decimal point), so callers defer to the underlying parser for those
 * cases.
 *
 * @param value The raw string entered by the user.
 */
export function countDecimalPlaces(value: string): number | null {
  const trimmed = value.trim();
  if (trimmed === "") return null;

  // Optional sign, integer part, optional fractional part. Plain decimal only
  // (no scientific notation, no thousands separators) to match the inputs the
  // `Float` decimal parser accepts.
  const match = /^[+-]?(\d*)(?:\.(\d*))?$/.exec(trimmed);
  if (!match) return null;

  const [, integerPart, fractionalPart] = match;
  // Reject strings with no digits at all (e.g. ".", "+", "-").
  if ((integerPart ?? "") === "" && (fractionalPart ?? "") === "") return null;

  return (fractionalPart ?? "").length;
}

/**
 * Validates that a raw amount string does not exceed the maximum supported
 * number of decimal places.
 *
 * @param value The raw string entered by the user.
 * @returns Validation result with a friendly error message when invalid.
 */
export function validateAmount(value: string): AmountDecimalsValidation {
  const decimals = countDecimalPlaces(value);

  if (decimals !== null && decimals > MAX_DECIMALS) {
    return {
      isValid: false,
      errorMessage: `Too many decimal places. A maximum of ${MAX_DECIMALS} decimal places is allowed.`,
    };
  }

  return {
    isValid: true,
    errorMessage: null,
  };
}
