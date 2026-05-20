import { render, screen } from "@testing-library/svelte";
import ValidOrdersSection from "$lib/components/deployment/ValidOrdersSection.svelte";
import type { ValidOrderDetail } from "$lib/types/order";

describe("validOrdersSection", () => {
  const mockvalidOrders: ValidOrderDetail[] = [
    {
      dotrain: "",
      name: "order1",
      details: {
        name: "Order 1",
        description: "Test order 1",
        short_description: "Short description 1",
      },
    },
    {
      dotrain: "",
      name: "order2",
      details: {
        name: "Order 2",
        description: "Test order 2",
        short_description: "Short description 2",
      },
    },
  ];

  it("should render correct number of OrderShortTile components", () => {
    render(ValidOrdersSection, { props: { orders: mockvalidOrders } });
    const orderTiles = screen.getAllByTestId("order-short-tile");
    expect(orderTiles).toHaveLength(mockvalidOrders.length);
  });
});
