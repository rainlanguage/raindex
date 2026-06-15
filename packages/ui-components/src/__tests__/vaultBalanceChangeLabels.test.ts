import { describe, expect, it } from "vitest";
import type {
  RaindexVaultBalanceChangeType,
  VaultBalanceChangeFilter,
} from "@rainlanguage/raindex";
import {
  VAULT_BALANCE_CHANGE_FILTER_LABELS,
  VAULT_BALANCE_CHANGE_LABELS,
  labelForVaultBalanceChangeType,
} from "../lib/utils/vaultBalanceChangeLabels";

describe("VAULT_BALANCE_CHANGE_LABELS", () => {
  it("maps each balance change type to its display label", () => {
    expect(VAULT_BALANCE_CHANGE_LABELS).toEqual({
      deposit: "Deposit",
      withdrawal: "Withdrawal",
      takeOrder: "Take order",
      clear: "Clear",
      clearBounty: "Clear Bounty",
      unknown: "Unknown",
    });
  });

  // Pin each entry individually so a single mutated value is caught precisely.
  it.each([
    ["deposit", "Deposit"],
    ["withdrawal", "Withdrawal"],
    ["takeOrder", "Take order"],
    ["clear", "Clear"],
    ["clearBounty", "Clear Bounty"],
    ["unknown", "Unknown"],
  ] as [RaindexVaultBalanceChangeType, string][])(
    "labels %s as %s",
    (type, label) => {
      expect(VAULT_BALANCE_CHANGE_LABELS[type]).toBe(label);
    },
  );
});

describe("VAULT_BALANCE_CHANGE_FILTER_LABELS", () => {
  it("maps each filterable balance change type to its display label", () => {
    expect(VAULT_BALANCE_CHANGE_FILTER_LABELS).toEqual({
      deposit: "Deposit",
      withdrawal: "Withdrawal",
      takeOrder: "Take order",
      clear: "Clear",
      clearBounty: "Clear Bounty",
    });
  });

  it.each([
    ["deposit", "Deposit"],
    ["withdrawal", "Withdrawal"],
    ["takeOrder", "Take order"],
    ["clear", "Clear"],
    ["clearBounty", "Clear Bounty"],
  ] as [VaultBalanceChangeFilter, string][])(
    "labels %s as %s",
    (type, label) => {
      expect(VAULT_BALANCE_CHANGE_FILTER_LABELS[type]).toBe(label);
    },
  );

  it("excludes the non-filterable 'unknown' type", () => {
    expect("unknown" in VAULT_BALANCE_CHANGE_FILTER_LABELS).toBe(false);
  });

  it("agrees with the full label map for every shared key", () => {
    for (const key of Object.keys(
      VAULT_BALANCE_CHANGE_FILTER_LABELS,
    ) as VaultBalanceChangeFilter[]) {
      expect(VAULT_BALANCE_CHANGE_FILTER_LABELS[key]).toBe(
        VAULT_BALANCE_CHANGE_LABELS[key],
      );
    }
  });
});

describe("labelForVaultBalanceChangeType", () => {
  it.each([
    ["deposit", "Deposit"],
    ["withdrawal", "Withdrawal"],
    ["takeOrder", "Take order"],
    ["clear", "Clear"],
    ["clearBounty", "Clear Bounty"],
    ["unknown", "Unknown"],
  ] as [RaindexVaultBalanceChangeType, string][])(
    "returns the mapped label for %s",
    (type, label) => {
      expect(labelForVaultBalanceChangeType(type)).toBe(label);
    },
  );

  it("returns the looked-up label, not the raw type, for a known type", () => {
    // Guards against the lookup being dropped so the raw type leaks through:
    // "takeOrder" must resolve to "Take order", which differs from the input.
    const result = labelForVaultBalanceChangeType("takeOrder");
    expect(result).toBe("Take order");
    expect(result).not.toBe("takeOrder");
  });

  it("falls back to the raw type when it is not in the label map", () => {
    // Exercises the `?? type` branch with a type absent from the map.
    const unmapped = "somethingElse" as RaindexVaultBalanceChangeType;
    expect(labelForVaultBalanceChangeType(unmapped)).toBe("somethingElse");
  });
});
