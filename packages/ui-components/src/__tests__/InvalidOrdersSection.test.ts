import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/svelte";
import InvalidOrdersSection from "../lib/components/deployment/InvalidOrdersSection.svelte";
import type { InvalidOrderDetail } from "$lib/types/order";

describe("InvalidOrdersSection", () => {
  const mockInvalidOrders: InvalidOrderDetail[] = [
    {
      name: "Order 1",
      error: "Invalid configuration",
    },
    {
      name: "Order 2",
      error: "Missing required field",
    },
  ];

  it("displays multiple invalid orders with their errors", () => {
    render(InvalidOrdersSection, {
      props: { ordersWithErrors: mockInvalidOrders },
    });

    expect(screen.getByText("Order 1")).toBeInTheDocument();
    expect(screen.getByText("Order 2")).toBeInTheDocument();
    expect(screen.getByText("Invalid configuration")).toBeInTheDocument();
    expect(screen.getByText("Missing required field")).toBeInTheDocument();
  });
});
