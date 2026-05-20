import { render, screen } from "@testing-library/svelte";
import { describe, it, expect } from "vitest";
import DropdownProperty from "../lib/components/DropdownProperty.svelte";

describe("DropdownProperty", () => {
  it("should render the component", () => {
    render(DropdownProperty, {
      key: "testKey",
      value: "testValue",
    });

    expect(screen.getByText("testKey")).toBeInTheDocument();
    expect(screen.getByText("testValue")).toBeInTheDocument();
  });
  it("should handle empty values", () => {
    render(DropdownProperty, {
      key: "emptyTest",
      value: "",
    });
    expect(screen.getByText("emptyTest")).toBeInTheDocument();
  });
});
