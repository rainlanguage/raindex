import type { Readable } from "svelte/store";
import type { Hex } from "viem";

export type Account = Readable<Hex | null>;
