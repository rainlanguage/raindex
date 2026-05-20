import { describe, it, expect } from "vitest";
import { render, fireEvent } from "@testing-library/svelte";
import DropdownRadio from "../lib/components/dropdown/DropdownRadio.svelte";
import { tick } from "svelte";

describe("DropdownRadio", () => {
  const options = {
    option1: { label: "Option 1" },
    option2: { label: "Option 2" },
    option3: { label: "Option 3" },
  };

  it("renders without crashing", () => {
    const { container } = render(DropdownRadio, { props: { options } });
    expect(container).toBeTruthy();
  });

  it("displays options when clicked", async () => {
    const { getByRole, getAllByRole } = render(DropdownRadio, {
      props: { options },
    });

    const button = getByRole("button");
    await fireEvent.click(button);

    const radioButtons = getAllByRole("radio");
    expect(radioButtons).toHaveLength(3);
  });

  it("selects an option when clicked", async () => {
    const { getByRole, getAllByRole } = render(DropdownRadio, {
      props: { options, value: undefined },
    });

    const button = getByRole("button");
    await fireEvent.click(button);

    const radioButtons = getAllByRole("radio");
    await fireEvent.click(radioButtons[0]);

    expect(radioButtons[0]).toBeChecked();
  });

  it("emits change event when option selected", async () => {
    let selectedValue;
    const { getByRole, getAllByRole, component } = render(DropdownRadio, {
      props: { options },
    });

    component.$on("change", (e) => {
      selectedValue = e.detail.value;
    });

    const button = getByRole("button");
    await fireEvent.click(button);

    const radioButtons = getAllByRole("radio");
    await fireEvent.click(radioButtons[0]);

    expect(selectedValue).toBe("option1");
  });

  it("closes dropdown after selection", async () => {
    const { getByRole, getAllByRole, getByTestId } = render(DropdownRadio, {
      props: { options },
    });

    const button = getByRole("button");
    await fireEvent.click(button);

    const dropdown = getByTestId("dropdown");
    expect(dropdown).toBeVisible();

    const radioButtons = getAllByRole("radio");
    await fireEvent.click(radioButtons[0]);

    expect(dropdown).not.toBeVisible();
  });

  it("sorts options alphabetically", async () => {
    const unsortedOptions = {
      c: { label: "C Option" },
      a: { label: "A Option" },
      b: { label: "B Option" },
    };

    const { getByRole, getAllByRole } = render(DropdownRadio, {
      props: { options: unsortedOptions },
    });

    const button = getByRole("button");
    await fireEvent.click(button);

    const radioButtons = getAllByRole("radio") as HTMLInputElement[];
    expect(radioButtons[0].value).toBe("a");
    expect(radioButtons[1].value).toBe("b");
    expect(radioButtons[2].value).toBe("c");
  });

  it("emits change event and closes dropdown when value changes programmatically", async () => {
    let selectedValue;
    const { component, getByRole, getByTestId } = render(DropdownRadio, {
      props: { options, value: undefined },
    });

    const button = getByRole("button");
    await fireEvent.click(button);

    const dropdown = getByTestId("dropdown");
    expect(dropdown).toBeVisible();

    component.$on("change", (e) => {
      selectedValue = e.detail.value;
    });

    component.$set({ value: "option1" });
    await tick();

    expect(selectedValue).toBe("option1");
    expect(dropdown).not.toBeVisible();
  });
});
