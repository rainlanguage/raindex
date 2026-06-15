import { render, cleanup, fireEvent } from "@testing-library/svelte";
import { describe, it, expect, afterEach } from "vitest";
import { writable } from "svelte/store";
import { get } from "svelte/store";
import ButtonScrub from "../lib/components/ButtonScrub.svelte";

describe("ButtonScrub Component", () => {
  afterEach(cleanup);

  it("toggles the scrub store on from false when clicked", async () => {
    const scrub = writable(false);
    const { getByTestId } = render(ButtonScrub, { props: { scrub } });
    await fireEvent.click(getByTestId("scrub-toggle"));
    expect(get(scrub)).toBe(true);
  });

  it("toggles the scrub store off from true when clicked", async () => {
    const scrub = writable(true);
    const { getByTestId } = render(ButtonScrub, { props: { scrub } });
    await fireEvent.click(getByTestId("scrub-toggle"));
    expect(get(scrub)).toBe(false);
  });

  it("flips back and forth on repeated clicks", async () => {
    const scrub = writable(false);
    const { getByTestId } = render(ButtonScrub, { props: { scrub } });
    const button = getByTestId("scrub-toggle");
    await fireEvent.click(button);
    await fireEvent.click(button);
    expect(get(scrub)).toBe(false);
  });

  it("reflects scrub state via aria-pressed", async () => {
    const scrub = writable(false);
    const { getByTestId } = render(ButtonScrub, { props: { scrub } });
    const button = getByTestId("scrub-toggle");
    expect(button).toHaveAttribute("aria-pressed", "false");
    await fireEvent.click(button);
    expect(button).toHaveAttribute("aria-pressed", "true");
  });
});
