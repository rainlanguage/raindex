import { getExplorerLink } from "../lib/services/getExplorerLink";

vi.mock("viem/chains", () => ({
  mainnet: {
    id: 999,
    blockExplorers: {
      default: {
        url: "https://etherscan.io",
      },
    },
  },
}));

describe("getExplorerLink", () => {
  it("should return the explorer link", () => {
    expect(getExplorerLink("0x123", 999, "tx")).toBe(
      "https://etherscan.io/tx/0x123",
    );
  });
  it("should return an empty string if the chain is not found", () => {
    expect(getExplorerLink("0x123", 1, "tx")).toBe("");
  });
});
