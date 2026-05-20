import { render, screen } from "@testing-library/svelte";
import { describe, it, expect } from "vitest";
import DeploymentSectionHeader from "../lib/components/deployment/DeploymentSectionHeader.svelte";
import type { ComponentProps } from "svelte";

export type DeploymentSectionHeaderComponentProps =
  ComponentProps<DeploymentSectionHeader>;

describe("DeploymentSectionHeader", () => {
  const defaultProps: DeploymentSectionHeaderComponentProps = {
    title: "Test Title",
    description: "Test Description",
  };

  it("renders title and description", () => {
    render(DeploymentSectionHeader, defaultProps);

    expect(screen.getByText("Test Title")).toBeInTheDocument();
    expect(screen.getByText("Test Description")).toBeInTheDocument();
  });

  it("renders title only", () => {
    render(DeploymentSectionHeader, { title: "Test Title" });

    expect(screen.getByText("Test Title")).toBeInTheDocument();
    expect(screen.queryByText("Test Description")).not.toBeInTheDocument();
  });

  it("renders description with markdown", () => {
    render(DeploymentSectionHeader, {
      title: "Test Title",
      description:
        "**Bold** and *italic* text\n\n[Link](https://www.example.com)",
    });

    const boldElement = screen.getByText("Bold");
    expect(boldElement).toBeInTheDocument();
    expect(boldElement.tagName).toBe("STRONG");

    const italicElement = screen.getByText("italic");
    expect(italicElement).toBeInTheDocument();
    expect(italicElement.tagName).toBe("EM");

    const linkElement = screen.getByRole("link", { name: /Link/i });
    expect(linkElement).toBeInTheDocument();
    expect(linkElement).toHaveAttribute("href", "https://www.example.com");
  });
});
