import { render, screen, within } from "@testing-library/svelte";
import { describe, it, expect, vi } from "vitest";
import PageHeader from "../lib/components/PageHeader.svelte";

vi.mock("../lib/utils/breadcrumbs", () => ({
  generateBreadcrumbs: vi.fn(),
}));

import { generateBreadcrumbs } from "../lib/utils/breadcrumbs";

describe("PageHeader.svelte", () => {
  const mockTitle = "Test Page";
  const mockPathname = "/test/path";
  const mockCrumbs = [
    { label: "Test", href: "/test" },
    { label: "Path", href: "/test/path" },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(generateBreadcrumbs).mockReturnValue(mockCrumbs);
  });

  it("renders the title correctly", () => {
    render(PageHeader, { props: { title: mockTitle, pathname: mockPathname } });

    expect(screen.getByTestId("breadcrumb-page-title")).toHaveTextContent(
      mockTitle,
    );
  });

  it("calls generateBreadcrumbs with the correct pathname", () => {
    render(PageHeader, { props: { title: mockTitle, pathname: mockPathname } });

    expect(generateBreadcrumbs).toHaveBeenCalledTimes(1);
    expect(generateBreadcrumbs).toHaveBeenCalledWith(mockPathname);
  });

  it("renders the generated breadcrumbs", () => {
    render(PageHeader, { props: { title: mockTitle, pathname: mockPathname } });

    const nav = screen.getByRole("navigation", {
      name: "Default breadcrumb example",
    });

    const links = within(nav).getAllByRole("link");
    expect(links.length).toBe(1 + mockCrumbs.length);

    expect(links[0]).toHaveAttribute("href", "/");

    mockCrumbs.forEach((crumb, index) => {
      expect(links[index + 1]).toHaveTextContent(crumb.label);
      expect(links[index + 1]).toHaveAttribute("href", crumb.href);
    });
  });

  it("handles empty breadcrumbs array", () => {
    vi.mocked(generateBreadcrumbs).mockReturnValue([]);
    render(PageHeader, { props: { title: mockTitle, pathname: mockPathname } });

    expect(screen.getByTestId("breadcrumb-page-title")).toHaveTextContent(
      mockTitle,
    );

    const nav = screen.getByRole("navigation", {
      name: "Default breadcrumb example",
    });
    const links = within(nav).getAllByRole("link");
    expect(links.length).toBe(1);
    expect(links[0]).toHaveAttribute("href", "/");
  });
});
