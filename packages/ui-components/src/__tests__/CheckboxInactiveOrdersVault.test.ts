import { render, fireEvent, screen } from "@testing-library/svelte";
import { get, writable, type Writable } from "svelte/store";
import { beforeEach, expect, test, describe } from "vitest";
import CheckboxInactiveOrdersVault from "../lib/components/CheckboxInactiveOrdersVault.svelte";

describe("CheckboxInactiveOrdersVault", () => {
  let hideInactiveOrdersVaults: Writable<boolean>;

  beforeEach(() => {
    hideInactiveOrdersVaults = writable(false);
  });

  test("renders correctly", () => {
    render(CheckboxInactiveOrdersVault, {
      props: {
        hideInactiveOrdersVaults,
      },
    });
    expect(
      screen.getByText("Hide vaults without active orders"),
    ).toBeInTheDocument();
  });

  test("checkbox defaults to unchecked", () => {
    render(CheckboxInactiveOrdersVault, {
      props: {
        hideInactiveOrdersVaults,
      },
    });

    const checkbox = screen.getByRole("checkbox");
    expect(checkbox).not.toBeChecked();
  });

  test("toggles store value when clicked", async () => {
    render(CheckboxInactiveOrdersVault, {
      props: {
        hideInactiveOrdersVaults,
      },
    });

    const checkbox = screen.getByRole("checkbox");
    expect(get(hideInactiveOrdersVaults)).toBe(false);

    await fireEvent.click(checkbox);
    expect(get(hideInactiveOrdersVaults)).toBe(true);

    await fireEvent.click(checkbox);
    expect(get(hideInactiveOrdersVaults)).toBe(false);
  });

  test("renders with correct test id", () => {
    render(CheckboxInactiveOrdersVault, {
      props: {
        hideInactiveOrdersVaults,
      },
    });

    expect(
      screen.getByTestId("inactive-orders-vault-checkbox"),
    ).toBeInTheDocument();
  });
});
