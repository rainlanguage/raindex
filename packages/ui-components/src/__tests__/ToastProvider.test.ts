import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/svelte";
import ToastProvider from "../lib/providers/toasts/ToastProvider.svelte";
import { writable, type Writable } from "svelte/store";
import type { ToastProps } from "../lib/types/toast";

vi.mock("../lib/providers/toasts/context", async () => ({
  setToastsContext: vi.fn(),
  getToastsContext: vi.fn(),
}));

import { setToastsContext } from "../lib/providers/toasts/context";

describe("ToastProvider", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should create and set up a writable store for toasts", () => {
    render(ToastProvider);

    expect(setToastsContext).toHaveBeenCalledTimes(1);
    const toastsArg = vi.mocked(setToastsContext).mock.calls[0][0];
    expect(toastsArg).toBeDefined();

    // Verify it's a writable store
    let value: ToastProps[] = [];
    const unsubscribe = toastsArg.subscribe((v) => {
      value = v;
    });
    expect(value).toEqual([]);
    unsubscribe();
  });

  it("should properly render toast notifications", () => {
    const mockToasts = writable<ToastProps[]>([
      {
        color: "green",
        message: "Test toast",
        type: "success",
        links: [],
      },
    ]);

    // Mock the context to return our test store
    vi.mocked(setToastsContext).mockImplementation(
      (store: Writable<ToastProps[]>) => {
        mockToasts.subscribe((value) => {
          store.set(value);
        });
      },
    );

    render(ToastProvider);

    expect(screen.getByRole("alert")).toBeInTheDocument();
    expect(screen.getByText("Test toast")).toBeInTheDocument();
    expect(screen.getByRole("alert")).toHaveTextContent("Test toast");
  });

  it("should render multiple toasts when present", () => {
    const mockToasts = writable<ToastProps[]>([
      {
        color: "green",
        message: "First toast",
        type: "success",
        links: [],
      },
      {
        color: "red",
        message: "Second toast",
        type: "error",
        links: [],
      },
    ]);

    // Mock the context to return our test store
    vi.mocked(setToastsContext).mockImplementation(
      (store: Writable<ToastProps[]>) => {
        mockToasts.subscribe((value) => {
          store.set(value);
        });
      },
    );

    render(ToastProvider);

    const alerts = screen.getAllByRole("alert");
    expect(alerts).toHaveLength(2);
    expect(screen.getByText("First toast")).toBeInTheDocument();
    expect(screen.getByText("Second toast")).toBeInTheDocument();
  });

  it("should render toasts with links when provided", () => {
    const mockToasts = writable<ToastProps[]>([
      {
        color: "green",
        message: "Test toast with link",
        type: "success",
        links: [
          {
            link: "https://example.com",
            label: "Example Link",
          },
        ],
      },
    ]);

    // Mock the context to return our test store
    vi.mocked(setToastsContext).mockImplementation(
      (store: Writable<ToastProps[]>) => {
        mockToasts.subscribe((value) => {
          store.set(value);
        });
      },
    );

    render(ToastProvider);

    expect(screen.getByText("Test toast with link")).toBeInTheDocument();
    const link = screen.getByText("Example Link");
    expect(link).toBeInTheDocument();
    expect(link).toHaveAttribute("href", "https://example.com");
    expect(link).toHaveAttribute("target", "_blank");
    expect(link).toHaveAttribute("rel", "noopener noreferrer");
  });
});
