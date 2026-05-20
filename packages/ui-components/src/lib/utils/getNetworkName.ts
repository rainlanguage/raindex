import * as chains from "viem/chains";

export function getNetworkName(chainId: number): string | undefined {
  const chain = Object.values(chains).find((chain) => chain.id === chainId);
  return chain?.name;
}

if (import.meta.vitest) {
  describe("getNetworkName", () => {
    it("should return the network name for a given chain id", () => {
      expect(getNetworkName(1)).toBe("Ethereum");
      expect(getNetworkName(137)).toBe("Polygon");
    });
  });
}
