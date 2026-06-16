import { describe, it, expect, beforeEach, vi } from "vitest";
import { get } from "svelte/store";

describe("scrub store", () => {
  beforeEach(() => {
    localStorage.clear();
    // Re-import a fresh module instance so the cached value is read from the
    // now-cleared localStorage rather than a value retained from a prior test.
    vi.resetModules();
  });

  it("defaults to false when nothing is persisted", async () => {
    const { scrub } = await import("../lib/storesGeneric/scrub");
    expect(get(scrub)).toBe(false);
  });

  it("persists the toggled value to localStorage under the 'scrub' key", async () => {
    const { scrub } = await import("../lib/storesGeneric/scrub");
    scrub.set(true);
    expect(localStorage.getItem("scrub")).toBe("true");
    scrub.set(false);
    expect(localStorage.getItem("scrub")).toBe("false");
  });

  it("rehydrates true from localStorage on load", async () => {
    localStorage.setItem("scrub", "true");
    const { scrub } = await import("../lib/storesGeneric/scrub");
    expect(get(scrub)).toBe(true);
  });

  it("falls back to false for corrupt persisted values", async () => {
    localStorage.setItem("scrub", "not-json");
    const { scrub } = await import("../lib/storesGeneric/scrub");
    expect(get(scrub)).toBe(false);
  });

  it("coerces any non-true persisted value to false", async () => {
    // A truthy-but-not-`true` JSON value must not enable scrubbing.
    localStorage.setItem("scrub", '"yes"');
    const { scrub } = await import("../lib/storesGeneric/scrub");
    expect(get(scrub)).toBe(false);
  });
});
