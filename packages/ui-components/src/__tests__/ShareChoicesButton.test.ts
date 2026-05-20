import { render, fireEvent, screen } from "@testing-library/svelte";
import { describe, it, expect, vi } from "vitest";
import ShareChoicesButton from "../lib/components/deployment/ShareChoicesButton.svelte";

describe("ShareChoicesButton", () => {
  const mockHandleShareChoices = vi.fn();

  it("calls handleShareChoices when clicked", async () => {
    render(ShareChoicesButton, {
      props: {
        handleShareChoices: mockHandleShareChoices,
      },
    });

    await fireEvent.click(screen.getByTestId("review-choices-button"));
    expect(mockHandleShareChoices).toHaveBeenCalledTimes(1);
  });

  it("shows and hides copied message when clicked", async () => {
    render(ShareChoicesButton, {
      props: {
        handleShareChoices: mockHandleShareChoices,
      },
    });

    // Message should not be visible initially
    expect(
      screen.queryByText("Shareable URL copied to clipboard"),
    ).not.toBeInTheDocument();

    // Click the button
    await fireEvent.click(screen.getByTestId("review-choices-button"));

    // Message should be visible
    expect(
      screen.getByText("Shareable URL copied to clipboard"),
    ).toBeInTheDocument();

    // Wait for 5 seconds
    await new Promise((resolve) => setTimeout(resolve, 5000));

    // Message should be gone
    expect(
      screen.queryByText("Shareable URL copied to clipboard"),
    ).not.toBeInTheDocument();
  });

  it("renders the share button correctly", () => {
    render(ShareChoicesButton, {
      props: {
        handleShareChoices: mockHandleShareChoices,
      },
    });

    expect(screen.getByText("Share these choices")).toBeInTheDocument();
    expect(screen.getByTestId("review-choices-button")).toBeInTheDocument();
  });
});
