import { render, screen } from "@testing-library/svelte";
import { describe, it, expect } from "vitest";
import { tick } from "svelte";
import LocalDbStatusBadge from "../lib/components/LocalDbStatusBadge.svelte";

describe("LocalDbStatusBadge", () => {
  it("renders the active state by default", () => {
    render(LocalDbStatusBadge);
    expect(screen.getByText("Active")).toBeInTheDocument();
  });

  it("updates when the status store changes", async () => {
    const { component } = render(LocalDbStatusBadge);
    component.$set({ status: "syncing" });
    await tick();
    expect(screen.getByText("Syncing")).toBeInTheDocument();

    component.$set({ status: "failure" });
    await tick();
    expect(screen.getByText("Failure")).toBeInTheDocument();
  });
});
