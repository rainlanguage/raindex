import { toHex } from "viem";

export const HEX_INPUT_REGEX = /^(0x)?([0-9a-f]+)?$/;

export const bigintStringToHex = (val: string) => toHex(BigInt(val));
