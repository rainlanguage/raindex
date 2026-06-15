import { cachedWritableStore } from "./cachedWritableStore";

/**
 * Global "scrub" toggle used for safe screenshotting of the GUI.
 *
 * When `true`, every {@link Sensitive} region (which wraps all rendered
 * addresses and transaction hashes via the `Hash` component) is masked with an
 * opaque grey box so that sharing a screenshot does not leak addresses/hashes.
 *
 * This is purely presentational: it never changes any on-chain data or the
 * underlying values held in memory, only how they are displayed.
 *
 * The value is persisted to localStorage under the `scrub` key so the masked
 * state survives reloads while a user is preparing screenshots.
 */
export const scrub = cachedWritableStore<boolean>(
  "scrub",
  false,
  (value) => JSON.stringify(value),
  (serialized) => {
    try {
      return JSON.parse(serialized) === true;
    } catch {
      return false;
    }
  },
);
