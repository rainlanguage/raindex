import { sortBy } from "lodash";
import type { UTCTimestamp } from "lightweight-charts";

/**
 * Filters out data points with duplicate timestamps, keeping only the first occurrence.
 *
 * @param data Array of data points with time and value properties
 * @returns A new array with only unique timestamps
 */
export const deduplicateByTime = <T extends { time: UTCTimestamp }>(
  data: T[],
): T[] => {
  const uniqueData: T[] = [];
  const seenTimes = new Set<UTCTimestamp>();

  for (const dataPoint of data) {
    if (!seenTimes.has(dataPoint.time)) {
      uniqueData.push(dataPoint);
      seenTimes.add(dataPoint.time);
    }
  }

  return uniqueData;
};

/**
 * Transforms and sorts data, ensuring unique timestamps.
 *
 * @param data The source data to transform
 * @param options Configuration object with transform functions
 * @returns Transformed, sorted, and deduplicated data
 */
export const transformAndSortData = <T>(
  data: T[],
  options: {
    valueTransform: (item: T) => number;
    timeTransform: (item: T) => UTCTimestamp;
  },
): Array<{ value: number; time: UTCTimestamp }> => {
  const { valueTransform, timeTransform } = options;

  const transformedData = data.map((d) => ({
    value: valueTransform(d),
    time: timeTransform(d),
  }));

  const sortedData = sortBy(transformedData, (d) => d.time);

  return deduplicateByTime(sortedData);
};

if (import.meta.vitest) {
  const { it, expect, describe } = import.meta.vitest;

  describe("deduplicateByTime", () => {
    it("should remove entries with duplicate timestamps", () => {
      const data = [
        { time: 100 as UTCTimestamp, value: 10 },
        { time: 200 as UTCTimestamp, value: 20 },
        { time: 200 as UTCTimestamp, value: 25 }, // Duplicate timestamp
        { time: 300 as UTCTimestamp, value: 30 },
      ];

      const result = deduplicateByTime(data);
      const expected = [
        { time: 100 as UTCTimestamp, value: 10 },
        { time: 200 as UTCTimestamp, value: 20 }, // First occurrence kept
        { time: 300 as UTCTimestamp, value: 30 },
      ];

      expect(result).toEqual(expected);
    });

    it("should handle multiple duplicate timestamps", () => {
      const data = [
        { time: 100 as UTCTimestamp, value: 10 },
        { time: 100 as UTCTimestamp, value: 15 }, // Duplicate
        { time: 100 as UTCTimestamp, value: 18 }, // Duplicate
        { time: 200 as UTCTimestamp, value: 20 },
      ];

      const result = deduplicateByTime(data);
      const expected = [
        { time: 100 as UTCTimestamp, value: 10 }, // Only first one kept
        { time: 200 as UTCTimestamp, value: 20 },
      ];

      expect(result).toEqual(expected);
    });

    it("should return original array if no duplicates", () => {
      const data = [
        { time: 100 as UTCTimestamp, value: 10 },
        { time: 200 as UTCTimestamp, value: 20 },
        { time: 300 as UTCTimestamp, value: 30 },
      ];

      const result = deduplicateByTime(data);
      expect(result).toEqual(data);

      expect(result).not.toBe(data);
    });

    it("should handle empty array", () => {
      const data: Array<{ time: UTCTimestamp; value: number }> = [];
      const result = deduplicateByTime(data);
      expect(result).toEqual([]);
    });
  });

  describe("transformAndSortData", () => {
    it("should transform, sort, and deduplicate data", () => {
      const rawData = [
        { timestamp: 3000, price: 300 },
        { timestamp: 1000, price: 100 },
        { timestamp: 2000, price: 200 },
        { timestamp: 2000, price: 250 }, // Duplicate timestamp
      ];

      const result = transformAndSortData(rawData, {
        valueTransform: (item) => item.price,
        timeTransform: (item) => item.timestamp as UTCTimestamp,
      });

      const expected = [
        { time: 1000 as UTCTimestamp, value: 100 },
        { time: 2000 as UTCTimestamp, value: 200 }, // First occurrence kept after sorting
        { time: 3000 as UTCTimestamp, value: 300 },
      ];

      expect(result).toEqual(expected);
    });

    it("should handle empty data array", () => {
      const rawData: Array<{ timestamp: number; price: number }> = [];

      const result = transformAndSortData(rawData, {
        valueTransform: (item) => item.price,
        timeTransform: (item) => item.timestamp as UTCTimestamp,
      });

      expect(result).toEqual([]);
    });
  });
}
