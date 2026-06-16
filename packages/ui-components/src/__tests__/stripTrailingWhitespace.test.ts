import { describe, it, expect } from "vitest";
import { stripTrailingWhitespace } from "../lib/utils/stripTrailingWhitespace";

describe("stripTrailingWhitespace", () => {
  it("removes trailing spaces and tabs from each line", () => {
    const input = "/* one */   \n_: int-add(1 2);\t\t";
    expect(stripTrailingWhitespace(input)).toBe("/* one */\n_: int-add(1 2);");
  });

  it("preserves leading whitespace (indentation) on every line", () => {
    const input = "  _a: 1; \n\t\t_b: 2;\t";
    expect(stripTrailingWhitespace(input)).toBe("  _a: 1;\n\t\t_b: 2;");
  });

  it("preserves internal blank lines but strips their whitespace", () => {
    const input = "a:1; \n   \n\t\nb:2;";
    expect(stripTrailingWhitespace(input)).toBe("a:1;\n\n\nb:2;");
  });

  it("removes trailing blank lines and the final newline", () => {
    const input = "a:1;\nb:2;\n\n   \n\t\n";
    expect(stripTrailingWhitespace(input)).toBe("a:1;\nb:2;");
  });

  it("normalises CRLF and CR line endings to LF", () => {
    const input = "a:1; \r\nb:2;\t\rc:3;  ";
    expect(stripTrailingWhitespace(input)).toBe("a:1;\nb:2;\nc:3;");
  });

  it("returns an empty string for whitespace-only input", () => {
    expect(stripTrailingWhitespace("   \n\t\n  \n")).toBe("");
  });

  it("leaves text with no trailing whitespace unchanged", () => {
    const input = "a:1;\nb:2;";
    expect(stripTrailingWhitespace(input)).toBe(input);
  });

  it("does not strip whitespace that is interior to a line", () => {
    const input = "_: int-add(1   2); ";
    expect(stripTrailingWhitespace(input)).toBe("_: int-add(1   2);");
  });
});
