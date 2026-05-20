import { describe, test, expect } from "vitest";
import { render } from "@testing-library/svelte";
import EditableSpan from "../lib/components/EditableSpan.svelte";

describe("Editable Span", () => {
  test("should show the correct value", async () => {
    const screen = render(EditableSpan, {
      displayValue: "123",
    });

    expect(screen.getByText("Block:")).toBeInTheDocument();
    expect(screen.getByTestId("editableSpan")).toHaveTextContent("123");

    screen.getByTestId("editableSpan").click();
    screen
      .getByTestId("editableSpan")
      .dispatchEvent(new KeyboardEvent("keydown", { key: "Enter" }));
  });
});
