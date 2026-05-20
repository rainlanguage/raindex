import { render, fireEvent, screen } from "@testing-library/svelte";
import { get, writable, type Writable } from "svelte/store";
import { beforeEach, expect, test, describe } from "vitest";
import CheckboxZeroBalanceVault from "../lib/components/CheckboxZeroBalanceVault.svelte";

describe("CheckboxZeroBalanceVault", () => {
  let hideZeroBalanceVaults: Writable<boolean>;

  beforeEach(() => {
    hideZeroBalanceVaults = writable(false);
  });

  test("renders correctly", () => {
    render(CheckboxZeroBalanceVault, {
      props: {
        hideZeroBalanceVaults,
      },
    });
    expect(screen.getByText("Hide empty vaults")).toBeInTheDocument();
  });

  test("toggles store value when clicked", async () => {
    render(CheckboxZeroBalanceVault, {
      props: {
        hideZeroBalanceVaults,
      },
    });

    const checkbox = screen.getByRole("checkbox");
    expect(get(hideZeroBalanceVaults)).toBe(false);

    await fireEvent.click(checkbox);
    expect(get(hideZeroBalanceVaults)).toBe(true);

    await fireEvent.click(checkbox);
    expect(get(hideZeroBalanceVaults)).toBe(false);
  });
});
