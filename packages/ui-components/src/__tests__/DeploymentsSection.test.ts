import { render, screen } from "@testing-library/svelte";
import { describe, it, expect } from "vitest";
import DeploymentsSection from "../lib/components/deployment/DeploymentsSection.svelte";

describe("DeploymentsSection", () => {
  it("renders provided deployments", () => {
    const mockDeployments = new Map([
      [
        "key1",
        {
          name: "Deployment 1",
          description: "Description 1",
          short_description: "Short 1",
        },
      ],
      [
        "key2",
        {
          name: "Deployment 2",
          description: "Description 2",
          short_description: "Short 2",
        },
      ],
    ]);

    render(DeploymentsSection, {
      props: {
        deployments: mockDeployments,
        orderName: "Test Strategy",
      },
    });

    expect(screen.getByText("Deployment 1")).toBeInTheDocument();
    expect(screen.getByText("Deployment 2")).toBeInTheDocument();
  });

  it("shows empty state when there are no deployments", () => {
    render(DeploymentsSection, {
      props: {
        deployments: [],
        orderName: "Test Strategy",
      },
    });

    expect(
      screen.getByText("No deployments found for this order."),
    ).toBeInTheDocument();
  });
});
