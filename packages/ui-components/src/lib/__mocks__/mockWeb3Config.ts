import { createConfig, http, fallback, type Config } from "@wagmi/core";
import { mock } from "@wagmi/connectors";
import { polygonAmoy } from "@wagmi/core/chains";

export const mockWeb3Config: Config = createConfig({
  multiInjectedProviderDiscovery: true,
  chains: [polygonAmoy],
  connectors: [
    mock({
      accounts: ["0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11"],
    }),
  ],
  transports: {
    [polygonAmoy.id]: fallback([
      http(),
      http("https://rpc-amoy.polygon.technology"),
    ]),
  },
});
