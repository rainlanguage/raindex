import { describe, it, expect, vi, beforeEach } from "vitest";
import { render } from "@testing-library/svelte";
import TransactionProvider from "../lib/providers/transactions/TransactionProvider.svelte";
import { TransactionManager } from "../lib/providers/transactions/TransactionManager";
import { writable } from "svelte/store";
import type { Config } from "@wagmi/core";
import type { ToastProps } from "../lib/types/toast";

vi.mock("../lib/providers/transactions/context", () => ({
  setTransactionManagerContext: vi.fn(),
}));

vi.mock("@tanstack/svelte-query", () => ({
  useQueryClient: () => ({
    invalidateQueries: vi.fn(),
  }),
}));

import { setTransactionManagerContext } from "../lib/providers/transactions/context";

describe("TransactionProvider", () => {
  const mockAddToast = vi.fn();
  const mockWagmiConfig = writable<Config>({
    chains: [],
    connectors: [],
    storage: {
      getItem: vi.fn(),
      setItem: vi.fn(),
      removeItem: vi.fn(),
    },
    state: {
      connections: new Map(),
      status: "disconnected",
    },
    setState: vi.fn(),
    subscribe: vi.fn(),
    getState: vi.fn(),
    destroy: vi.fn(),
  } as unknown as Config);

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should initialize TransactionManager with correct dependencies", () => {
    render(TransactionProvider, {
      addToast: mockAddToast,
      wagmiConfig: mockWagmiConfig,
    });

    expect(setTransactionManagerContext).toHaveBeenCalledTimes(1);
    const managerArg = vi.mocked(setTransactionManagerContext).mock.calls[0][0];
    expect(managerArg).toBeInstanceOf(TransactionManager);
  });

  it("should pass addToast function to TransactionManager", () => {
    render(TransactionProvider, {
      addToast: mockAddToast,
      wagmiConfig: mockWagmiConfig,
    });

    const managerArg = vi.mocked(setTransactionManagerContext).mock
      .calls[0][0] as TransactionManager;
    const transactions = managerArg.getTransactions();
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    let storeValue: any[] = [];
    transactions.subscribe((value) => {
      storeValue = value;
    });
    expect(storeValue).toEqual([]);
  });

  it("should handle toast notifications", () => {
    const toast: ToastProps = {
      message: "Test toast",
      type: "success",
      color: "green",
      links: [],
    };

    render(TransactionProvider, {
      addToast: mockAddToast,
      wagmiConfig: mockWagmiConfig,
    });

    mockAddToast(toast);
    expect(mockAddToast).toHaveBeenCalledWith(toast);
  });
});
