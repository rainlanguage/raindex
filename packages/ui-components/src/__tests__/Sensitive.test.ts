import { render, cleanup } from "@testing-library/svelte";
import { describe, it, expect, afterEach } from "vitest";
import { writable } from "svelte/store";
import { tick } from "svelte";
import Sensitive from "../lib/components/Sensitive.svelte";

describe("Sensitive Component", () => {
  afterEach(cleanup);

  it("does not render a mask when scrub is off", () => {
    const scrub = writable(false);
    const { queryByTestId } = render(Sensitive, { props: { scrub } });
    expect(queryByTestId("sensitive-mask")).not.toBeInTheDocument();
  });

  it("renders an opaque mask overlay when scrub is on", () => {
    const scrub = writable(true);
    const { getByTestId } = render(Sensitive, { props: { scrub } });
    const mask = getByTestId("sensitive-mask");
    expect(mask).toBeInTheDocument();
    // The overlay must be a solid (non-transparent) box that covers the slot.
    expect(mask).toHaveClass("absolute", "inset-0", "bg-gray-400");
    expect(mask).toHaveAttribute("aria-hidden", "true");
  });

  it("hides the underlying content visually when scrub is on", () => {
    const scrub = writable(true);
    const { getByTestId } = render(Sensitive, { props: { scrub } });
    // Content stays in the DOM (presentational only) but is made invisible.
    expect(getByTestId("sensitive-content")).toHaveClass("invisible");
  });

  it("shows the underlying content when scrub is off", () => {
    const scrub = writable(false);
    const { getByTestId } = render(Sensitive, { props: { scrub } });
    expect(getByTestId("sensitive-content")).not.toHaveClass("invisible");
  });

  it("reacts to the store toggling at runtime", async () => {
    const scrub = writable(false);
    const { queryByTestId, getByTestId } = render(Sensitive, {
      props: { scrub },
    });
    expect(queryByTestId("sensitive-mask")).not.toBeInTheDocument();

    scrub.set(true);
    await tick();
    expect(getByTestId("sensitive-mask")).toBeInTheDocument();
    expect(getByTestId("sensitive-content")).toHaveClass("invisible");

    scrub.set(false);
    await tick();
    expect(queryByTestId("sensitive-mask")).not.toBeInTheDocument();
    expect(getByTestId("sensitive-content")).not.toHaveClass("invisible");
  });
});
