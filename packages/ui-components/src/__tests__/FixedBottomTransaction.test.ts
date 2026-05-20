import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/svelte";
import FixedBottomTransaction from "../lib/components/transactions/FixedBottomTransaction.svelte";
import { TransactionManager } from "../lib/providers/transactions/TransactionManager";
import { writable } from "svelte/store";
import type {
  Transaction,
  TransactionStoreState,
} from "../lib/models/Transaction";
import {
  TransactionStatusMessage,
  TransactionName,
  TransactionStoreErrorMessage,
} from "../lib/types/transaction";
import { useTransactions } from "../lib/providers/transactions/useTransactions";
import { getStatusEmoji } from "$lib/components/transactions/getStatusEmoji";

vi.mock("../lib/providers/transactions/useTransactions", () => ({
  useTransactions: vi.fn(),
}));

describe("FixedBottomTransaction", () => {
  let mockTransactionManager: TransactionManager;
  let mockTransactionsStore: ReturnType<typeof writable<Transaction[]>>;

  beforeEach(() => {
    vi.clearAllMocks();

    mockTransactionsStore = writable<Transaction[]>([]);

    mockTransactionManager = {
      transactions: vi.fn().mockReturnValue(mockTransactionsStore),
    } as unknown as TransactionManager;

    vi.mocked(useTransactions).mockReturnValue({
      manager: mockTransactionManager,
      transactions: mockTransactionsStore,
    });
  });

  it("should not render anything when there are no transactions", () => {
    const { container } = render(FixedBottomTransaction);
    expect(container.innerHTML).toBe("");
  });

  it("should render the latest transaction when transactions exist", () => {
    const mockTransactions = [
      {
        state: writable<TransactionStoreState>({
          status: TransactionStatusMessage.IDLE,
          name: TransactionName.REMOVAL,
          links: [],
        }),
      },
      {
        state: writable<TransactionStoreState>({
          status: TransactionStatusMessage.SUCCESS,
          name: TransactionName.REMOVAL,
          links: [
            {
              label: "View on Explorer",
              link: "https://etherscan.io/tx/0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            },
          ],
        }),
      },
    ] as unknown as Transaction[];

    mockTransactionsStore.set(mockTransactions);

    render(FixedBottomTransaction);

    // Should render the latest transaction (second one)
    expect(screen.getByText(TransactionName.REMOVAL)).toBeInTheDocument();
    expect(
      screen.getByText(TransactionStatusMessage.SUCCESS),
    ).toBeInTheDocument();
    expect(
      screen.getByText(getStatusEmoji(TransactionStatusMessage.SUCCESS)),
    ).toBeInTheDocument();
    expect(screen.getByText("View on Explorer")).toBeInTheDocument();
  });

  it("should have proper mobile-only styling with fixed positioning", () => {
    const mockTransactions = [
      {
        state: writable<TransactionStoreState>({
          status: TransactionStatusMessage.PENDING_RECEIPT,
          name: TransactionName.REMOVAL,
          links: [],
        }),
      },
    ] as unknown as Transaction[];

    mockTransactionsStore.set(mockTransactions);

    const { container } = render(FixedBottomTransaction);

    const transactionElement = container.querySelector("div");
    expect(transactionElement).toHaveClass(
      "fixed",
      "bottom-0",
      "left-0",
      "right-0",
      "z-40",
      "lg:hidden",
    );
  });

  it("should display transaction details correctly", () => {
    const mockTransactions = [
      {
        state: writable<TransactionStoreState>({
          status: TransactionStatusMessage.PENDING_RECEIPT,
          name: TransactionName.REMOVAL,
          links: [
            {
              label: "Etherscan",
              link: "https://etherscan.io/tx/0x123",
            },
            {
              label: "Block Explorer",
              link: "https://explorer.com/tx/0x456",
            },
          ],
          errorDetails: undefined,
        }),
      },
    ] as unknown as Transaction[];

    mockTransactionsStore.set(mockTransactions);

    render(FixedBottomTransaction);

    expect(screen.getByText(TransactionName.REMOVAL)).toBeInTheDocument();
    expect(
      screen.getByText(TransactionStatusMessage.PENDING_RECEIPT),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        getStatusEmoji(TransactionStatusMessage.PENDING_RECEIPT),
      ),
    ).toBeInTheDocument();
    expect(screen.getByText("Etherscan")).toBeInTheDocument();
    expect(screen.getByText("Block Explorer")).toBeInTheDocument();
  });

  it("should display error details when present", () => {
    const mockTransactions = [
      {
        state: writable<TransactionStoreState>({
          status: TransactionStatusMessage.ERROR,
          name: TransactionName.REMOVAL,
          links: [],
          errorDetails:
            "Transaction failed due to insufficient gas" as TransactionStoreErrorMessage,
        }),
      },
    ] as unknown as Transaction[];

    mockTransactionsStore.set(mockTransactions);

    render(FixedBottomTransaction);

    expect(screen.getByText(TransactionName.REMOVAL)).toBeInTheDocument();
    expect(
      screen.getByText("Transaction failed due to insufficient gas"),
    ).toBeInTheDocument();
    expect(
      screen.getByText(getStatusEmoji(TransactionStatusMessage.ERROR)),
    ).toBeInTheDocument();
  });

  it("should limit links to maximum of 2", () => {
    const mockTransactions = [
      {
        state: writable<TransactionStoreState>({
          status: TransactionStatusMessage.SUCCESS,
          name: TransactionName.REMOVAL,
          links: [
            { label: "Link 1", link: "https://example1.com" },
            { label: "Link 2", link: "https://example2.com" },
            { label: "Link 3", link: "https://example3.com" },
            { label: "Link 4", link: "https://example4.com" },
          ],
        }),
      },
    ] as unknown as Transaction[];

    mockTransactionsStore.set(mockTransactions);

    render(FixedBottomTransaction);

    expect(screen.getByText("Link 1")).toBeInTheDocument();
    expect(screen.getByText("Link 2")).toBeInTheDocument();
    expect(screen.queryByText("Link 3")).not.toBeInTheDocument();
    expect(screen.queryByText("Link 4")).not.toBeInTheDocument();
  });

  it("should dismiss transaction when close button is clicked", async () => {
    const mockTransactions = [
      {
        state: writable<TransactionStoreState>({
          status: TransactionStatusMessage.SUCCESS,
          name: TransactionName.REMOVAL,
          links: [],
        }),
      },
    ] as unknown as Transaction[];

    mockTransactionsStore.set(mockTransactions);

    const { container } = render(FixedBottomTransaction);

    expect(screen.getByText(TransactionName.REMOVAL)).toBeInTheDocument();

    const closeButton = screen.getByRole("button", {
      name: "Dismiss transaction",
    });
    expect(closeButton).toBeInTheDocument();

    await fireEvent.click(closeButton);

    await waitFor(() => {
      expect(container.innerHTML).toBe("");
    });
  });

  it("should reset dismiss state when new transaction appears", async () => {
    const { container } = render(FixedBottomTransaction);

    // Start with one transaction
    const firstTransaction = [
      {
        state: writable<TransactionStoreState>({
          status: TransactionStatusMessage.SUCCESS,
          name: TransactionName.REMOVAL,
          links: [],
        }),
      },
    ] as unknown as Transaction[];

    mockTransactionsStore.set(firstTransaction);

    await waitFor(() => {
      expect(screen.getByText(TransactionName.REMOVAL)).toBeInTheDocument();
    });

    // Dismiss it
    const closeButton = screen.getByRole("button", {
      name: "Dismiss transaction",
    });
    await fireEvent.click(closeButton);

    await waitFor(() => {
      expect(container.innerHTML).toBe("");
    });

    // Add a new transaction
    const secondTransaction = [
      {
        state: writable<TransactionStoreState>({
          status: TransactionStatusMessage.PENDING_RECEIPT,
          name: TransactionName.REMOVAL,
          links: [],
        }),
      },
    ] as unknown as Transaction[];

    mockTransactionsStore.set(secondTransaction);

    // Should show the new transaction (dismiss state reset)
    await waitFor(() => {
      expect(screen.getByText(TransactionName.REMOVAL)).toBeInTheDocument();
    });
  });

  it("should update when transactions change", async () => {
    const { container } = render(FixedBottomTransaction);

    expect(container.innerHTML).toBe("");

    // Add first transaction
    mockTransactionsStore.set([
      {
        state: writable<TransactionStoreState>({
          status: TransactionStatusMessage.IDLE,
          name: TransactionName.REMOVAL,
          links: [],
        }),
      },
    ] as unknown as Transaction[]);

    await waitFor(() => {
      expect(screen.getByText(TransactionName.REMOVAL)).toBeInTheDocument();
    });

    // Add second transaction (should show the latest one)
    mockTransactionsStore.set([
      {
        state: writable<TransactionStoreState>({
          status: TransactionStatusMessage.IDLE,
          name: TransactionName.REMOVAL,
          links: [],
        }),
      },
      {
        state: writable<TransactionStoreState>({
          status: TransactionStatusMessage.SUCCESS,
          name: "Latest Transaction" as TransactionName,
          links: [],
        }),
      },
    ] as unknown as Transaction[]);

    await waitFor(() => {
      expect(screen.getByText("Latest Transaction")).toBeInTheDocument();
      expect(
        screen.queryByText(TransactionName.REMOVAL),
      ).not.toBeInTheDocument();
    });

    // Remove all transactions
    mockTransactionsStore.set([]);

    await waitFor(() => {
      expect(container.innerHTML).toBe("");
    });
  });

  it("should get transactions from the useTransactions hook", () => {
    render(FixedBottomTransaction);
    expect(useTransactions).toHaveBeenCalled();
  });

  it("should handle transactions without links", () => {
    const mockTransactions = [
      {
        state: writable<TransactionStoreState>({
          status: TransactionStatusMessage.PENDING_SUBGRAPH,
          name: TransactionName.REMOVAL,
          links: [],
        }),
      },
    ] as unknown as Transaction[];

    mockTransactionsStore.set(mockTransactions);

    render(FixedBottomTransaction);

    expect(screen.getByText(TransactionName.REMOVAL)).toBeInTheDocument();
    expect(
      screen.getByText(TransactionStatusMessage.PENDING_SUBGRAPH),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        getStatusEmoji(TransactionStatusMessage.PENDING_SUBGRAPH),
      ),
    ).toBeInTheDocument();

    // Should not render links section
    const linkContainer = screen.queryByRole("link");
    expect(linkContainer).not.toBeInTheDocument();
  });

  it("should have proper accessibility attributes", () => {
    const mockTransactions = [
      {
        state: writable<TransactionStoreState>({
          status: TransactionStatusMessage.SUCCESS,
          name: TransactionName.REMOVAL,
          links: [],
        }),
      },
    ] as unknown as Transaction[];

    mockTransactionsStore.set(mockTransactions);

    render(FixedBottomTransaction);

    const closeButton = screen.getByRole("button", {
      name: "Dismiss transaction",
    });
    expect(closeButton).toHaveAttribute("aria-label", "Dismiss transaction");
  });

  it("should open links in new tab with proper attributes", () => {
    const mockTransactions = [
      {
        state: writable<TransactionStoreState>({
          status: TransactionStatusMessage.SUCCESS,
          name: TransactionName.REMOVAL,
          links: [
            {
              label: "External Link",
              link: "https://example.com",
            },
          ],
        }),
      },
    ] as unknown as Transaction[];

    mockTransactionsStore.set(mockTransactions);

    render(FixedBottomTransaction);

    const link = screen.getByRole("link", { name: "External Link" });
    expect(link).toHaveAttribute("href", "https://example.com");
    expect(link).toHaveAttribute("target", "_blank");
    expect(link).toHaveAttribute("rel", "noopener noreferrer");
  });
});
