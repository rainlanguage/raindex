import { render } from "@testing-library/svelte";
import Hash from "../lib/components/Hash.svelte";
import { describe, it, expect } from "vitest";
import userEvent from "@testing-library/user-event";

describe("Hash Component", () => {
  it("renders with shortened hash display", () => {
    const { getByText } = render(Hash, {
      props: {
        value: "abcdef1234567890",
        type: 1, // HashType.Wallet
        shorten: true,
        sliceLen: 5,
      },
    });
    expect(getByText("abcde...67890")).toBeInTheDocument();
  });

  it("renders full hash when shorten is false", () => {
    const { getByText } = render(Hash, {
      props: {
        value: "abcdef1234567890",
        type: 1,
        shorten: false,
      },
    });

    expect(getByText("abcdef1234567890")).toBeInTheDocument();
  });

  it("copies hash to clipboard and shows copied message", async () => {
    const user = userEvent.setup();

    const { getByRole, findByText } = render(Hash, {
      props: {
        value: "abcdef1234567890",
        type: 1,
        copyOnClick: true,
      },
    });

    const button = getByRole("button");
    await user.click(button);
    expect(await findByText("Copied to clipboard")).toBeInTheDocument();
    const clipboardText = await navigator.clipboard.readText();
    expect(clipboardText).toBe("abcdef1234567890");
  });

  it("does not copy to clipboard if copyOnClick is false", async () => {
    const user = userEvent.setup();

    const { getByRole } = render(Hash, {
      props: {
        value: "abcdef1234567890",
        type: 1,
        copyOnClick: false,
      },
    });

    const button = getByRole("button");
    await user.click(button);
    const clipboardText = await navigator.clipboard.readText();
    expect(clipboardText).toBe("");
  });
});
