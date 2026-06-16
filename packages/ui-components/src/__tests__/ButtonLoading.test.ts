import { render, screen } from "@testing-library/svelte";
import { describe, it, expect } from "vitest";
import ButtonLoading from "../lib/components/ButtonLoading.svelte";

describe("ButtonLoading", () => {
  it("should render a button with a spinner when loading is true", () => {
    render(ButtonLoading, {
      loading: true,
    });
    expect(screen.getByTestId("spinner")).toBeInTheDocument();
  });

  it("should disable the button when loading is true", () => {
    render(ButtonLoading, {
      loading: true,
    });
    const button = screen.getByRole("button");
    expect(button).toBeDisabled();
  });

  it("should not render a spinner when loading is false", () => {
    render(ButtonLoading, {
      loading: false,
    });
    expect(screen.queryByTestId("spinner")).not.toBeInTheDocument();
  });
});
