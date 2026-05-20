import { render, screen } from "@testing-library/svelte";
import { describe, it, expect, vi } from "vitest";
import DeploymentTile from "../lib/components/deployment/DeploymentTile.svelte";

// Mock the goto function
vi.mock("$app/navigation", () => ({
  goto: vi.fn(),
}));

describe("DeploymentTile", () => {
  const mockProps = {
    orderName: "test-order",
    key: "test-key",
    name: "Test Deployment",
    description: "This is a test deployment description",
  };

  it("renders the deployment name and description", () => {
    render(DeploymentTile, mockProps);

    expect(screen.getByText("Test Deployment")).toBeInTheDocument();
    expect(
      screen.getByText("This is a test deployment description"),
    ).toBeInTheDocument();
  });
});
