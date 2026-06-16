import { describe, it, expect } from "vitest";
import {
  validateAmount,
  countDecimalPlaces,
  MAX_DECIMALS,
} from "$lib/services/validateAmount";

const EXPECTED_ERROR =
  "Too many decimal places. A maximum of 18 decimal places is allowed.";

describe("countDecimalPlaces", () => {
  it("counts zero decimal places for an integer", () => {
    expect(countDecimalPlaces("123")).toBe(0);
  });

  it("counts the digits after the decimal point", () => {
    expect(countDecimalPlaces("0.001511607327103594")).toBe(18);
    expect(countDecimalPlaces("0.0015116073271035947")).toBe(19);
  });

  it("ignores integer-part digits when counting", () => {
    // 18 integer digits, 3 fractional digits -> 3
    expect(countDecimalPlaces("123456789012345678.123")).toBe(3);
  });

  it("counts trailing zeros as decimal places", () => {
    expect(countDecimalPlaces("1.10")).toBe(2);
  });

  it("handles a leading sign", () => {
    expect(countDecimalPlaces("-0.0015116073271035947")).toBe(19);
    expect(countDecimalPlaces("+1.5")).toBe(1);
  });

  it("trims surrounding whitespace before counting", () => {
    expect(countDecimalPlaces("  1.234  ")).toBe(3);
  });

  it("returns null for empty / non-decimal input so the parser decides", () => {
    expect(countDecimalPlaces("")).toBeNull();
    expect(countDecimalPlaces("   ")).toBeNull();
    expect(countDecimalPlaces("abc")).toBeNull();
    expect(countDecimalPlaces("1e-19")).toBeNull();
    expect(countDecimalPlaces("1,234.5")).toBeNull();
    expect(countDecimalPlaces(".")).toBeNull();
    expect(countDecimalPlaces("-")).toBeNull();
  });
});

describe("validateAmount", () => {
  it("exposes the documented maximum of 18 decimal places", () => {
    expect(MAX_DECIMALS).toBe(18);
  });

  it("accepts exactly 18 decimal places (the issue's working value)", () => {
    expect(validateAmount("0.001511607327103594")).toEqual({
      isValid: true,
      errorMessage: null,
    });
  });

  it("rejects 19 decimal places (the issue's failing value) with a friendly message", () => {
    expect(validateAmount("0.0015116073271035947")).toEqual({
      isValid: false,
      errorMessage: EXPECTED_ERROR,
    });
  });

  it("rejects a negative value with too many decimals", () => {
    expect(validateAmount("-0.0000000000000000001")).toEqual({
      isValid: false,
      errorMessage: EXPECTED_ERROR,
    });
  });

  it("accepts integers and short decimals", () => {
    expect(validateAmount("100")).toEqual({
      isValid: true,
      errorMessage: null,
    });
    expect(validateAmount("0.0005")).toEqual({
      isValid: true,
      errorMessage: null,
    });
  });

  it("treats empty / non-plain-decimal input as valid (deferred to the parser)", () => {
    // These are not "too many decimals"; the underlying Float parser handles them.
    expect(validateAmount("")).toEqual({ isValid: true, errorMessage: null });
    expect(validateAmount("abc")).toEqual({
      isValid: true,
      errorMessage: null,
    });
    expect(validateAmount("1e-30")).toEqual({
      isValid: true,
      errorMessage: null,
    });
  });

  it("does not count integer digits toward the decimal limit", () => {
    // 25 integer digits but only 2 fractional digits is fine.
    expect(validateAmount("1234567890123456789012345.99")).toEqual({
      isValid: true,
      errorMessage: null,
    });
  });
});
