import { render, screen, fireEvent } from "@testing-library/svelte";
import { describe, it, expect, vi } from "vitest";
import DropdownCheckbox from "../lib/components/dropdown/DropdownCheckbox.svelte";

describe("DropdownCheckbox", () => {
  const defaultOptions = {
    option1: "Option 1",
    option2: "Option 2",
    option3: "Option 3",
  };

  describe("Rendering", () => {
    it("renders with label", () => {
      render(DropdownCheckbox, {
        props: { options: defaultOptions, label: "Test Label" },
      });

      expect(screen.getByText("Test Label")).toBeInTheDocument();
    });

    it("renders dropdown button", () => {
      render(DropdownCheckbox, {
        props: { options: defaultOptions },
      });

      expect(
        screen.getByTestId("dropdown-checkbox-button"),
      ).toBeInTheDocument();
    });

    it('shows "Select items" when nothing selected', () => {
      render(DropdownCheckbox, {
        props: { options: defaultOptions },
      });

      expect(screen.getByTestId("dropdown-checkbox-button")).toHaveTextContent(
        "Select items",
      );
    });

    it("shows count when items selected", async () => {
      render(DropdownCheckbox, {
        props: {
          options: defaultOptions,
          value: { option1: "Option 1", option2: "Option 2" },
        },
      });

      expect(screen.getByTestId("dropdown-checkbox-button")).toHaveTextContent(
        "2 items",
      );
    });

    it("shows allLabel when all selected", () => {
      render(DropdownCheckbox, {
        props: {
          options: defaultOptions,
          value: { ...defaultOptions },
          allLabel: "All Items",
        },
      });

      expect(screen.getByTestId("dropdown-checkbox-button")).toHaveTextContent(
        "All Items",
      );
    });

    it("shows emptyMessage when options empty", async () => {
      render(DropdownCheckbox, {
        props: { options: {}, emptyMessage: "No items available" },
      });

      const button = screen.getByTestId("dropdown-checkbox-button");
      await fireEvent.click(button);

      expect(screen.getByText("No items available")).toBeInTheDocument();
    });
  });

  describe("Interactions", () => {
    it("toggles individual item on", async () => {
      const handleChange = vi.fn();
      const { component } = render(DropdownCheckbox, {
        props: { options: defaultOptions, value: {} },
      });

      component.$on("change", handleChange);

      const button = screen.getByTestId("dropdown-checkbox-button");
      await fireEvent.click(button);

      const option1Label = screen.getByText("Option 1").closest("label");
      if (option1Label) await fireEvent.click(option1Label);

      expect(handleChange).toHaveBeenCalledWith(
        expect.objectContaining({
          detail: { option1: "Option 1" },
        }),
      );
    });

    it("toggles individual item off", async () => {
      const handleChange = vi.fn();
      const { component } = render(DropdownCheckbox, {
        props: { options: defaultOptions, value: { option1: "Option 1" } },
      });

      component.$on("change", handleChange);

      const button = screen.getByTestId("dropdown-checkbox-button");
      await fireEvent.click(button);

      const option1Label = screen.getByText("Option 1").closest("label");
      if (option1Label) await fireEvent.click(option1Label);

      expect(handleChange).toHaveBeenCalledWith(
        expect.objectContaining({
          detail: {},
        }),
      );
    });

    it('toggles all items via "All items" checkbox', async () => {
      const handleChange = vi.fn();
      const { component } = render(DropdownCheckbox, {
        props: { options: defaultOptions, value: {}, allLabel: "All Items" },
      });

      component.$on("change", handleChange);

      const button = screen.getByTestId("dropdown-checkbox-button");
      await fireEvent.click(button);

      const allItemsLabel = screen.getByText("All Items").closest("label");
      if (allItemsLabel) await fireEvent.click(allItemsLabel);

      expect(handleChange).toHaveBeenCalledWith(
        expect.objectContaining({
          detail: defaultOptions,
        }),
      );
    });

    it("dispatches change event with updated value", async () => {
      const handleChange = vi.fn();
      const { component } = render(DropdownCheckbox, {
        props: { options: defaultOptions, value: {} },
      });

      component.$on("change", handleChange);

      const button = screen.getByTestId("dropdown-checkbox-button");
      await fireEvent.click(button);

      const option1Label = screen.getByText("Option 1").closest("label");
      if (option1Label) await fireEvent.click(option1Label);

      expect(handleChange).toHaveBeenCalledWith(
        expect.objectContaining({
          detail: { option1: "Option 1" },
        }),
      );
    });
  });

  describe("Props", () => {
    it("onlyTitle=true hides secondary text (keys)", async () => {
      render(DropdownCheckbox, {
        props: { options: defaultOptions, onlyTitle: true },
      });

      const button = screen.getByTestId("dropdown-checkbox-button");
      await fireEvent.click(button);

      expect(screen.getByText("Option 1")).toBeInTheDocument();
      expect(screen.queryByText("option1")).not.toBeInTheDocument();
    });

    it("onlyTitle=false shows secondary text (keys)", async () => {
      render(DropdownCheckbox, {
        props: { options: defaultOptions, onlyTitle: false },
      });

      const button = screen.getByTestId("dropdown-checkbox-button");
      await fireEvent.click(button);

      expect(screen.getByText("Option 1")).toBeInTheDocument();
      expect(screen.getByText("option1")).toBeInTheDocument();
    });

    it('showAllLabel=false hides "All items" option', async () => {
      render(DropdownCheckbox, {
        props: {
          options: defaultOptions,
          showAllLabel: false,
          allLabel: "All Items",
        },
      });

      const button = screen.getByTestId("dropdown-checkbox-button");
      await fireEvent.click(button);

      expect(screen.queryByText("All Items")).not.toBeInTheDocument();
    });
  });
});
