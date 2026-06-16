import dayjs from "dayjs";
import bigIntSupport from "dayjs/plugin/bigIntSupport";
import localizedFormat from "dayjs/plugin/localizedFormat";
import type { UTCTimestamp } from "lightweight-charts";
import utc from "dayjs/plugin/utc";
dayjs.extend(bigIntSupport);
dayjs.extend(localizedFormat);
dayjs.extend(utc);

export const TIME_DELTA_24_HOURS = 60 * 60 * 24;
export const TIME_DELTA_48_HOURS = TIME_DELTA_24_HOURS * 2;
export const TIME_DELTA_7_DAYS = TIME_DELTA_24_HOURS * 7;
export const TIME_DELTA_30_DAYS = TIME_DELTA_24_HOURS * 30;
export const TIME_DELTA_1_YEAR = TIME_DELTA_24_HOURS * 365;

export function dateTimestamp(date: Date): number {
  return Math.floor(date.getTime() / 1000);
}

export function formatTimestampSecondsAsLocal(timestampSeconds: bigint) {
  return dayjs(timestampSeconds * BigInt("1000"))
    .utc()
    .format("L LT");
}

export function timestampSecondsToUTCTimestamp(timestampSeconds: bigint) {
  return dayjs(timestampSeconds * BigInt("1000")).unix() as UTCTimestamp;
}

/**
 * Method to put a timeout on a promise, throws the exception if promise is not settled within the time
 * @param promise - The original promise
 * @param time - The time in ms
 * @param exception - The exception to reject with if time runs out before original promise settlement
 * @returns A new promise that gets settled with initial promise settlement or rejected with exception value
 * if the time runs out before the main promise settlement
 */
export async function promiseTimeout<T>(
  promise: Promise<T>,
  time: number,
  exception: unknown,
): Promise<T> {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let timeout: any;
  return Promise.race([
    promise,
    new Promise(
      (_resolve, reject) => (timeout = setTimeout(reject, time, exception)),
    ) as Promise<T>,
  ]).finally(() => clearTimeout(timeout));
}

if (import.meta.vitest) {
  const { describe, it, expect, vi } = import.meta.vitest;

  describe("Date and timestamp utilities", () => {
    describe("formatTimestampSecondsAsLocal", () => {
      it("converts timestamp to local format", () => {
        const result = formatTimestampSecondsAsLocal(BigInt("1672531200")); // Jan 1, 2023 12:00 AM
        expect(result).toBe("01/01/2023 12:00 AM");
      });
    });

    describe("timestampSecondsToUTCTimestamp", () => {
      it("converts bigint timestamp to UTCTimestamp", () => {
        const result = timestampSecondsToUTCTimestamp(BigInt("1672531200"));
        expect(result).toBe(1672531200);
      });
    });
  });

  describe("promiseTimeout", () => {
    it("resolves when promise resolves before timeout", async () => {
      const testValue = "test";
      const promise = Promise.resolve(testValue);
      const result = await promiseTimeout(promise, 100, new Error("Timeout"));
      expect(result).toBe(testValue);
    });

    it("rejects when promise times out", async () => {
      const promise = new Promise((resolve) => setTimeout(resolve, 200));
      const exception = new Error("Timeout");

      await expect(promiseTimeout(promise, 100, exception)).rejects.toThrow(
        exception,
      );
    });

    it("rejects when original promise rejects", async () => {
      const error = new Error("Original rejection");
      const promise = Promise.reject(error);

      await expect(
        promiseTimeout(promise, 100, new Error("Timeout")),
      ).rejects.toThrow(error);
    });

    it("clears timeout after promise resolution", async () => {
      vi.spyOn(global, "clearTimeout");
      const promise = Promise.resolve("test");
      await promiseTimeout(promise, 100, new Error("Timeout"));

      expect(clearTimeout).toHaveBeenCalled();
    });

    it("clears timeout after promise rejection", async () => {
      vi.spyOn(global, "clearTimeout");
      const promise = Promise.reject(new Error("Original rejection"));

      try {
        await promiseTimeout(promise, 100, new Error("Timeout"));
      } catch {
        // Ignore the error
      }

      expect(clearTimeout).toHaveBeenCalled();
    });
  });
  describe("dateTimestamp", () => {
    it("should get date timestamp in seconds", () => {
      const date = new Date(2022, 1, 16, 17, 32, 11, 168);
      const result = dateTimestamp(date);
      const expected = Math.floor(date.getTime() / 1000);

      expect(result).toEqual(expected);
    });
  });
}
