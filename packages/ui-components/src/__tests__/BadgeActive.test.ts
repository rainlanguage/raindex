import { render, screen } from "@testing-library/svelte";
import BadgeActive from "../lib/components/BadgeActive.svelte";
import { describe, it, expect } from "vitest";

describe("BadgeActive", () => {
  it('shows "Active" text when active is true', () => {
    render(BadgeActive, { props: { active: true } });
    expect(screen.getByText("Active")).toBeInTheDocument();
  });

  it('shows "Inactive" text when active is false', () => {
    render(BadgeActive, { props: { active: false } });
    expect(screen.getByText("Inactive")).toBeInTheDocument();
  });

  it("uses green color for active state", () => {
    render(BadgeActive, { props: { active: true } });
    const badge = screen.getByText("Active");
    expect(badge.className).toContain("bg-green");
  });

  it("uses yellow color for inactive state", () => {
    render(BadgeActive, { props: { active: false } });
    const badge = screen.getByText("Inactive");
    expect(badge.className).toContain("bg-yellow");
  });
});
