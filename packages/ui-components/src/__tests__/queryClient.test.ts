import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  queryClient,
  invalidateTanstackQueries,
} from "../lib/queries/queryClient";
import { QueryClient } from "@tanstack/svelte-query";

describe("queryClient module", () => {
  describe("queryClient instance", () => {
    it("creates a QueryClient instance with correct configuration", () => {
      expect(queryClient).toBeInstanceOf(QueryClient);
    });
  });

  describe("invalidateTanstackQueries", () => {
    let testQueryClient: QueryClient;

    beforeEach(() => {
      testQueryClient = new QueryClient();
      vi.spyOn(testQueryClient, "invalidateQueries");
    });

    afterEach(() => {
      vi.restoreAllMocks();
    });

    it("calls invalidateQueries with the correct parameters", async () => {
      const queryKey = ["test", "query"];

      await invalidateTanstackQueries(testQueryClient, queryKey);

      expect(testQueryClient.invalidateQueries).toHaveBeenCalledTimes(1);
      expect(testQueryClient.invalidateQueries).toHaveBeenCalledWith({
        queryKey,
        refetchType: "all",
        exact: false,
      });
    });

    it("works with nested query keys", async () => {
      const queryKey = ["orders", "details", "123"];

      await invalidateTanstackQueries(testQueryClient, queryKey);

      expect(testQueryClient.invalidateQueries).toHaveBeenCalledWith({
        queryKey,
        refetchType: "all",
        exact: false,
      });
    });

    it("works with single-item query keys", async () => {
      const queryKey = ["globalData"];

      await invalidateTanstackQueries(testQueryClient, queryKey);

      expect(testQueryClient.invalidateQueries).toHaveBeenCalledWith({
        queryKey,
        refetchType: "all",
        exact: false,
      });
    });

    it("invalidates all matching queries due to exact: false", async () => {
      // Setup some test queries
      const parentKey = ["orders"];
      const childKey1 = ["orders", "123"];
      const childKey2 = ["orders", "456"];
      const unrelatedKey = ["products"];

      vi.spyOn(testQueryClient, "refetchQueries");

      await testQueryClient.prefetchQuery({
        queryKey: parentKey,
        queryFn: () => "parent data",
      });

      await testQueryClient.prefetchQuery({
        queryKey: childKey1,
        queryFn: () => "child1 data",
      });

      await testQueryClient.prefetchQuery({
        queryKey: childKey2,
        queryFn: () => "child2 data",
      });

      await testQueryClient.prefetchQuery({
        queryKey: unrelatedKey,
        queryFn: () => "unrelated data",
      });

      const unrelatedQueryStateBefore =
        testQueryClient.getQueryState(unrelatedKey);

      await invalidateTanstackQueries(testQueryClient, parentKey);

      expect(testQueryClient.invalidateQueries).toHaveBeenCalledWith({
        queryKey: parentKey,
        refetchType: "all",
        exact: false,
      });

      expect(testQueryClient.refetchQueries).toHaveBeenCalled();

      const unrelatedQueryStateAfter =
        testQueryClient.getQueryState(unrelatedKey);
      expect(unrelatedQueryStateAfter?.dataUpdatedAt).toBe(
        unrelatedQueryStateBefore?.dataUpdatedAt,
      );
    });

    it("handles potential errors gracefully", async () => {
      const queryKey = ["errorTest"];
      vi.spyOn(testQueryClient, "invalidateQueries").mockImplementationOnce(
        () => {
          throw new Error("Simulated invalidateQueries error");
        },
      );

      await expect(
        invalidateTanstackQueries(testQueryClient, queryKey),
      ).rejects.toThrow("Failed to refresh data.");
      expect(testQueryClient.invalidateQueries).toHaveBeenCalledWith({
        queryKey,
        refetchType: "all",
        exact: false,
      });
    });

    it("handles empty query key array", async () => {
      const queryKey: string[] = [];
      await invalidateTanstackQueries(testQueryClient, queryKey);
      expect(testQueryClient.invalidateQueries).toHaveBeenCalledWith({
        queryKey,
        refetchType: "all",
        exact: false,
      });
    });

    it("handles query key with undefined or null values", async () => {
      const queryKey = ["test", undefined, "key"];
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      await invalidateTanstackQueries(testQueryClient, queryKey as any);
      expect(testQueryClient.invalidateQueries).toHaveBeenCalledWith({
        queryKey: queryKey,
        refetchType: "all",
        exact: false,
      });

      const queryKeyWithNull = ["test", null, "key"];
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      await invalidateTanstackQueries(testQueryClient, queryKeyWithNull as any);
      expect(testQueryClient.invalidateQueries).toHaveBeenCalledWith({
        queryKey: queryKeyWithNull,
        refetchType: "all",
        exact: false,
      });
    });
  });
});
