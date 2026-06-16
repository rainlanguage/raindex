import { render, fireEvent } from "@testing-library/svelte";
import { describe, it, expect } from "vitest";
import InputTokenAmount from "$lib/components/input/InputTokenAmount.svelte";
import { Float } from "@rainlanguage/raindex";

vi.mock("@rainlanguage/raindex", async (importOriginal) => ({
  ...(await importOriginal()),
}));

describe("InputTokenAmount", () => {
  it("should handle empty input", async () => {
    const { getByRole, component } = render(InputTokenAmount, {
      props: { value: Float.parse("0").value },
    });
    const input = getByRole("textbox");

    await fireEvent.input(input, { target: { value: "" } });
    expect(component.$$.ctx[component.$$.props.value].format().value).toBe("0");
  });

  it("should handle invalid input", async () => {
    const { getByRole, component } = render(InputTokenAmount, {
      props: { value: Float.parse("0").value },
    });
    const input = getByRole("textbox");

    await fireEvent.input(input, { target: { value: "abc" } });
    expect(component.$$.ctx[component.$$.props.value].format().value).toBe("0");
  });

  it("should handle maxValue prop", async () => {
    const { getByText, component } = render(InputTokenAmount, {
      props: {
        maxValue: Float.parse("1").value,
        value: Float.parse("0").value,
      },
    });
    const maxButton = getByText("MAX");

    await fireEvent.click(maxButton);
    expect(component.$$.ctx[component.$$.props.value].format().value).toBe("1");
  });

  it("shows a friendly error and resets the value when more than 18 decimals are entered", async () => {
    const { getByRole, queryByTestId, component } = render(InputTokenAmount, {
      props: { value: Float.parse("0").value },
    });
    const input = getByRole("textbox");

    // 19 decimal places, one more than the maximum.
    await fireEvent.input(input, {
      target: { value: "0.0015116073271035947" },
    });

    const error = queryByTestId("decimals-error");
    expect(error).not.toBeNull();
    expect(error?.textContent?.trim()).toBe(
      "Too many decimal places. A maximum of 18 decimal places is allowed.",
    );
    // The invalid value must not propagate; it is reset to zero.
    expect(component.$$.ctx[component.$$.props.value].format().value).toBe("0");
  });

  it("accepts exactly 18 decimals without showing an error", async () => {
    const { getByRole, queryByTestId, component } = render(InputTokenAmount, {
      props: { value: Float.parse("0").value },
    });
    const input = getByRole("textbox");

    // 18 decimal places, exactly the maximum.
    await fireEvent.input(input, {
      target: { value: "0.001511607327103594" },
    });

    expect(queryByTestId("decimals-error")).toBeNull();
    expect(component.$$.ctx[component.$$.props.value].format().value).toBe(
      "0.001511607327103594",
    );
  });

  it("clears the decimals error once a valid value replaces an invalid one", async () => {
    const { getByRole, queryByTestId } = render(InputTokenAmount, {
      props: { value: Float.parse("0").value },
    });
    const input = getByRole("textbox");

    await fireEvent.input(input, {
      target: { value: "0.0015116073271035947" },
    });
    expect(queryByTestId("decimals-error")).not.toBeNull();

    await fireEvent.input(input, { target: { value: "0.5" } });
    expect(queryByTestId("decimals-error")).toBeNull();
  });
});
