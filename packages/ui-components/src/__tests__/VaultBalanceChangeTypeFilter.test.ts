import { render, screen, fireEvent } from "@testing-library/svelte";
import { describe, it, expect } from "vitest";
import VaultBalanceChangeTypeFilter from "../lib/components/VaultBalanceChangeTypeFilter.svelte";
import { VAULT_BALANCE_CHANGE_FILTER_LABELS } from "../lib/utils/vaultBalanceChangeLabels";

describe("VaultBalanceChangeTypeFilter", () => {
  it("renders the filter dropdown", () => {
    render(VaultBalanceChangeTypeFilter);

    expect(screen.getByText("Change Type")).toBeInTheDocument();
    expect(screen.getByTestId("dropdown-checkbox-button")).toBeInTheDocument();
  });

  it('shows "Select items" when nothing selected', () => {
    render(VaultBalanceChangeTypeFilter);

    expect(screen.getByTestId("dropdown-checkbox-button")).toHaveTextContent(
      "Select items",
    );
  });

  it("renders all filter options in dropdown", async () => {
    render(VaultBalanceChangeTypeFilter);

    const button = screen.getByTestId("dropdown-checkbox-button");
    await fireEvent.click(button);

    const options = screen.getAllByTestId("dropdown-checkbox-option");
    expect(options.length).toBeGreaterThanOrEqual(5);

    expect(
      screen.getByText(VAULT_BALANCE_CHANGE_FILTER_LABELS.deposit),
    ).toBeInTheDocument();
    expect(
      screen.getByText(VAULT_BALANCE_CHANGE_FILTER_LABELS.withdrawal),
    ).toBeInTheDocument();
    expect(
      screen.getByText(VAULT_BALANCE_CHANGE_FILTER_LABELS.takeOrder),
    ).toBeInTheDocument();
    expect(
      screen.getByText(VAULT_BALANCE_CHANGE_FILTER_LABELS.clear),
    ).toBeInTheDocument();
    expect(
      screen.getByText(VAULT_BALANCE_CHANGE_FILTER_LABELS.clearBounty),
    ).toBeInTheDocument();
  });

  it('shows "All types" when all filters selected', async () => {
    render(VaultBalanceChangeTypeFilter);

    const button = screen.getByTestId("dropdown-checkbox-button");
    await fireEvent.click(button);

    const allTypesCheckbox = screen.getByText("All types").closest("label");
    if (allTypesCheckbox) {
      await fireEvent.click(allTypesCheckbox);
    }

    expect(screen.getByTestId("dropdown-checkbox-button")).toHaveTextContent(
      "All types",
    );
  });

  it("shows item count when some filters selected", async () => {
    render(VaultBalanceChangeTypeFilter);

    const button = screen.getByTestId("dropdown-checkbox-button");
    await fireEvent.click(button);

    const withdrawalLabel = screen
      .getByText(VAULT_BALANCE_CHANGE_FILTER_LABELS.withdrawal)
      .closest("label");
    const depositLabel = screen
      .getByText(VAULT_BALANCE_CHANGE_FILTER_LABELS.deposit)
      .closest("label");

    if (withdrawalLabel) await fireEvent.click(withdrawalLabel);
    if (depositLabel) await fireEvent.click(depositLabel);

    expect(screen.getByTestId("dropdown-checkbox-button")).toHaveTextContent(
      "2 items",
    );
  });
});
