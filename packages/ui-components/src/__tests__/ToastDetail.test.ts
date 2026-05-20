import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import ToastDetail from "../lib/components/ToastDetail.svelte";
import type { ToastProps } from "../lib/types/toast";
import { useToasts } from "../lib/providers/toasts/useToasts";
import { writable } from "svelte/store";

// Mock the useToasts hook
vi.mock("../lib/providers/toasts/useToasts", () => ({
  useToasts: vi.fn(),
}));

const mockRemoveToast = vi.fn();

describe("ToastDetail", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(useToasts).mockReturnValue({
      removeToast: mockRemoveToast,
      toasts: writable([]),
      addToast: vi.fn(),
      errToast: vi.fn(),
    });
  });

  it("should render toast message", () => {
    const toast: ToastProps = {
      color: "green",
      message: "Test message",
      type: "success",
      links: [],
    };

    render(ToastDetail, { toast, i: 0 });
    expect(screen.getByText("Test message")).toBeInTheDocument();
  });

  it("should render success icon for success type", () => {
    const toast: ToastProps = {
      color: "green",
      message: "Success message",
      type: "success",
      links: [],
    };

    render(ToastDetail, { toast, i: 0 });
    expect(screen.getByTestId("success-icon")).toBeInTheDocument();
  });

  it("should render error icon for error type", () => {
    const toast: ToastProps = {
      color: "red",
      message: "Error message",
      type: "error",
      links: [],
    };

    render(ToastDetail, { toast, i: 0 });
    expect(screen.getByTestId("error-icon")).toBeInTheDocument();
  });

  it("should render links when provided", () => {
    const toast: ToastProps = {
      color: "green",
      message: "Message with links",
      type: "success",
      links: [
        {
          link: "https://example.com",
          label: "Example Link",
        },
        {
          link: "https://test.com",
          label: "Test Link",
        },
      ],
    };

    render(ToastDetail, { toast, i: 0 });

    const links = screen.getAllByRole("link");
    expect(links).toHaveLength(2);

    expect(links[0]).toHaveAttribute("href", "https://example.com");
    expect(links[0]).toHaveAttribute("target", "_blank");
    expect(links[0]).toHaveAttribute("rel", "noopener noreferrer");
    expect(links[0]).toHaveTextContent("Example Link");

    expect(links[1]).toHaveAttribute("href", "https://test.com");
    expect(links[1]).toHaveAttribute("target", "_blank");
    expect(links[1]).toHaveAttribute("rel", "noopener noreferrer");
    expect(links[1]).toHaveTextContent("Test Link");
  });

  it("should call removeToast when close button is clicked", async () => {
    const toast: ToastProps = {
      color: "green",
      message: "Test message",
      type: "success",
      links: [],
    };

    render(ToastDetail, { toast, i: 1 });

    const closeButton = screen.getByRole("button");
    await fireEvent.click(closeButton);

    expect(mockRemoveToast).toHaveBeenCalledWith(1);
  });

  it("should render detail when provided", () => {
    const toast: ToastProps = {
      color: "red",
      message: "Error message",
      type: "error",
      detail: "Details about what caused the error",
    };

    render(ToastDetail, { toast, i: 0 });
    expect(
      screen.getByText("Details about what caused the error"),
    ).toBeInTheDocument();
  });
});
