import { describe, expect, it } from "vitest";
import { bigintStringToPercentage } from "../lib/utils/number";

describe("bigintStringToPercentage", () => {
  it("scales the value by 100 to express it as a percentage", () => {
    // 12345 at 4 decimals is 1.2345; the *100 scaling expresses that as the
    // percentage 123.45.
    expect(bigintStringToPercentage("12345", 4)).toBe("123.45");
  });

  it("formats whole-number percentages with no decimal point", () => {
    // 5 at 1 decimal is 0.5 => 50%. formatUnits yields "50" (no "."),
    // so the truncation branch is skipped and the string is returned as-is.
    expect(bigintStringToPercentage("5", 1)).toBe("50");
  });

  it("returns zero for a zero value", () => {
    expect(bigintStringToPercentage("0", 18)).toBe("0");
  });

  it("uses valueDecimals to position the decimal point", () => {
    // Same digits, different decimals => different magnitude.
    expect(bigintStringToPercentage("12345", 4)).toBe("123.45");
    expect(bigintStringToPercentage("12345", 6)).toBe("1.2345");
  });

  it("truncates the fraction to the requested number of digits", () => {
    // raw percentage is "123.45"; keeping 1 fractional digit => "123.4".
    expect(bigintStringToPercentage("12345", 4, 1)).toBe("123.4");
  });

  it("keeps more fractional digits when finalDecimalsDigits is larger than the fraction", () => {
    // raw percentage "33.3333"; asking for 8 digits cannot invent digits.
    expect(bigintStringToPercentage("333333", 6, 8)).toBe("33.3333");
  });

  it("drops the decimal point entirely when finalDecimalsDigits is 0", () => {
    // raw percentage "123.45"; 0 fractional digits keeps the integer part
    // only, with no trailing dot.
    expect(bigintStringToPercentage("12345", 4, 0)).toBe("123");
  });

  it("defaults finalDecimals to valueDecimals when finalDecimalsDigits is omitted", () => {
    // raw percentage "1234.56"; default keeps up to valueDecimals (4) digits
    // so the full fraction survives.
    expect(bigintStringToPercentage("123456", 4)).toBe("1234.56");
  });

  it("does not corrupt a non-fractional result when finalDecimalsDigits is 0", () => {
    // formatUnits("50") has no ".", so indexOf is -1 and the truncation
    // branch is skipped: the full "50" is returned.
    expect(bigintStringToPercentage("5", 1, 0)).toBe("50");
  });
});
