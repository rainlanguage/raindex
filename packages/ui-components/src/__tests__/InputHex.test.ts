import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/svelte";
import InputHex from "../lib/components/input/InputHex.svelte";

describe("InputHex", () => {
  it("renders an input element", () => {
    render(InputHex);
    expect(screen.getByRole("textbox")).toBeTruthy();
  });

  it("initializes with empty string when no value provided", () => {
    render(InputHex);
    const input = screen.getByRole("textbox") as HTMLInputElement;
    expect(input.value).toBe("");
  });

  it("displays hex value when bigint is provided", () => {
    render(InputHex, { props: { value: 255n } });
    const input = screen.getByRole("textbox") as HTMLInputElement;
    expect(input.value).toBe("0xff");
  });
});
