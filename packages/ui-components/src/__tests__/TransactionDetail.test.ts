import { describe, it, expect } from "vitest";
import { render, screen, waitFor } from "@testing-library/svelte";
import TransactionDetail from "../lib/components/transactions/TransactionDetail.svelte";
import { writable } from "svelte/store";
import type { TransactionStoreState } from "../lib/models/Transaction";
import {
  TransactionName,
  TransactionStatusMessage,
  TransactionStoreErrorMessage,
} from "../lib/types/transaction";

describe("TransactionDetail", () => {
  it("should render different status emojis for different states", () => {
    const statuses = [
      { status: TransactionStatusMessage.PENDING_RECEIPT, emoji: "🔄" },
      { status: TransactionStatusMessage.PENDING_SUBGRAPH, emoji: "📊" },
      { status: TransactionStatusMessage.SUCCESS, emoji: "✅" },
      { status: TransactionStatusMessage.ERROR, emoji: "❌" },
    ] as const;

    statuses.forEach(({ status, emoji }) => {
      const state = writable<TransactionStoreState>({
        name: TransactionName.REMOVAL,
        status,
        links: [],
      });

      const { container } = render(TransactionDetail, { state });
      expect(container.textContent).toContain(emoji);
    });
  });

  it("should render links from the links array", () => {
    const mockLinks = [
      {
        label: "View on Explorer 1",
        link: "https://etherscan.io/tx/0xlink1",
      },
      {
        label: "View on Explorer 2",
        link: "https://polygonscan.com/tx/0xlink2",
      },
    ];
    const state = writable<TransactionStoreState>({
      name: TransactionName.REMOVAL,
      status: TransactionStatusMessage.PENDING_RECEIPT,
      links: mockLinks,
    });

    render(TransactionDetail, { state });

    mockLinks.forEach((mockLink) => {
      const linkElement = screen.getByRole("link", { name: mockLink.label });
      expect(linkElement).toBeInTheDocument();
      expect(linkElement).toHaveAttribute("href", mockLink.link);
      expect(linkElement).toHaveAttribute("target", "_blank");
      expect(linkElement).toHaveAttribute("rel", "noopener noreferrer");
    });
  });

  it("should render no links if the links array is empty", () => {
    const state = writable<TransactionStoreState>({
      name: TransactionName.REMOVAL,
      status: TransactionStatusMessage.PENDING_RECEIPT,
      links: [],
    });

    render(TransactionDetail, { state });

    const linkElements = screen.queryAllByRole("link");
    expect(linkElements.length).toBe(0);
  });

  it("should update when the state changes", async () => {
    const state = writable<TransactionStoreState>({
      name: TransactionName.REMOVAL,
      status: TransactionStatusMessage.PENDING_RECEIPT,
      links: [{ label: "Initial Link", link: "https://example.com/initial" }],
    });

    render(TransactionDetail, { state });

    expect(screen.getByText(TransactionName.REMOVAL)).toBeInTheDocument();
    expect(
      screen.getByText(`🔄 ${TransactionStatusMessage.PENDING_RECEIPT}`),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Initial Link" }),
    ).toBeInTheDocument();

    state.update((current) => ({
      ...current,
      status: TransactionStatusMessage.PENDING_SUBGRAPH,
      links: [{ label: "Updated Link", link: "https://example.com/updated" }],
    }));

    await waitFor(() => {
      expect(screen.getByText(TransactionName.REMOVAL)).toBeInTheDocument();
      expect(
        screen.getByText(`📊 ${TransactionStatusMessage.PENDING_SUBGRAPH}`),
      ).toBeInTheDocument();
      expect(
        screen.queryByRole("link", { name: "Initial Link" }),
      ).not.toBeInTheDocument();
      expect(
        screen.getByRole("link", { name: "Updated Link" }),
      ).toBeInTheDocument();
    });

    state.update((current) => ({
      ...current,
      status: TransactionStatusMessage.SUCCESS,
      links: [],
    }));

    await waitFor(() => {
      expect(screen.getByText(TransactionName.REMOVAL)).toBeInTheDocument();
      expect(
        screen.getByText(`✅ ${TransactionStatusMessage.SUCCESS}`),
      ).toBeInTheDocument();
      expect(
        screen.queryByRole("link", { name: "Updated Link" }),
      ).not.toBeInTheDocument();
    });
  });

  it("should handle error status with errorDetails if provided", () => {
    const errorDetails = "Something went terribly wrong.";
    const state = writable<TransactionStoreState>({
      status: TransactionStatusMessage.ERROR,
      name: TransactionName.REMOVAL,
      links: [],
      errorDetails: errorDetails as TransactionStoreErrorMessage,
    });

    render(TransactionDetail, { state });

    expect(screen.getByText(TransactionName.REMOVAL)).toBeInTheDocument();
    expect(
      screen.getByText(`❌ ${TransactionStatusMessage.ERROR}`),
    ).toBeInTheDocument();
    expect(screen.getByText(errorDetails)).toBeInTheDocument();
  });
});
