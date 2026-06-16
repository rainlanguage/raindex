import { render, fireEvent } from "@testing-library/svelte";
import { describe, it, expect, vi } from "vitest";
import Checkbox from "../lib/components/checkbox/Checkbox.svelte";
import { tick } from "svelte";

describe("Checkbox Component", () => {
  it("renders with default props", () => {
    const { getByTestId } = render(Checkbox);
    expect(getByTestId("checkbox")).toBeTruthy();
  });

  it("renders with label", () => {
    const label = "Test Label";
    const { getByText } = render(Checkbox, { props: { label } });
    expect(getByText(label)).toBeTruthy();
  });

  it("initializes with checked state", () => {
    const { getByTestId } = render(Checkbox, { props: { checked: true } });
    const checkbox = getByTestId("checkbox") as HTMLInputElement;
    expect(checkbox.checked).toBe(true);
  });

  it("handles change event", async () => {
    const mockChange = vi.fn();
    const { getByTestId, component } = render(Checkbox);

    component.$on("change", mockChange);
    const checkbox = getByTestId("checkbox");

    await fireEvent.click(checkbox);
    expect(mockChange).toHaveBeenCalledTimes(1);
    expect(mockChange.mock.calls[0][0].detail).toBe(true);

    await fireEvent.click(checkbox);
    expect(mockChange).toHaveBeenCalledTimes(2);
    expect(mockChange.mock.calls[1][0].detail).toBe(false);
  });

  it("updates checked state when clicked", async () => {
    const { getByTestId } = render(Checkbox);
    const checkbox = getByTestId("checkbox") as HTMLInputElement;

    expect(checkbox.checked).toBe(false);
    await fireEvent.click(checkbox);
    expect(checkbox.checked).toBe(true);
  });

  it("binds to checked prop", async () => {
    const checked = false;
    const { getByTestId, component } = render(Checkbox, { props: { checked } });
    const checkbox = getByTestId("checkbox") as HTMLInputElement;

    expect(checkbox.checked).toBe(false);

    // Use tick to wait for the next update cycle
    component.$set({ checked: true });
    await tick();

    expect(checkbox.checked).toBe(true);
  });
});
