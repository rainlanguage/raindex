import type { LocalDbStatus } from "@rainlanguage/raindex";

/**
 * The minimal shape required to aggregate a top-level localdb status: anything
 * carrying a per-chain/per-raindex {@link LocalDbStatus} and an optional error
 * message (e.g. `NetworkSyncStatus` or `RaindexSyncStatus`).
 */
export interface AggregableSyncStatus {
  status: LocalDbStatus;
  error?: string;
}

/**
 * Collapses many per-chain / per-raindex sync statuses into the single
 * top-level localdb status shown in the sidebar badge.
 *
 * Priority is `failure` > `syncing` > `active`:
 * - If any status is `failure`, the result is `failure` (carrying that
 *   status's error).
 * - Else if any status is `syncing`, the result is `syncing`.
 * - Else the result is `active`.
 *
 * An empty input (nothing configured) is treated as `active`.
 */
export function aggregateLocalDbStatus(statuses: AggregableSyncStatus[]): {
  status: LocalDbStatus;
  error: string | undefined;
} {
  const failure = statuses.find((s) => s.status === "failure");
  if (failure) {
    return { status: "failure", error: failure.error };
  }

  if (statuses.some((s) => s.status === "syncing")) {
    return { status: "syncing", error: undefined };
  }

  return { status: "active", error: undefined };
}
