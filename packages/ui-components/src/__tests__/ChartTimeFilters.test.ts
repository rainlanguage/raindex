import { render, fireEvent, screen } from "@testing-library/svelte";
import { get, writable } from "svelte/store";
import { test, expect } from "vitest";
import ChartTimeFiltersTest from "./ChartTimeFilters.test.svelte";

const TIME_DELTA_24_HOURS = 60 * 60 * 24;
const TIME_DELTA_7_DAYS = TIME_DELTA_24_HOURS * 7;
const TIME_DELTA_30_DAYS = TIME_DELTA_24_HOURS * 30;
const TIME_DELTA_1_YEAR = TIME_DELTA_24_HOURS * 365;

test("initial timeDelta is set to 1 year", async () => {
  const timeDeltaStore = writable(TIME_DELTA_1_YEAR);

  render(ChartTimeFiltersTest, { timeDeltaStore });

  const yearButton = screen.getByText("1 Year");
  expect(yearButton).toBeDisabled();
  expect(get(timeDeltaStore)).toBe(TIME_DELTA_1_YEAR);
});

test("clicking 30 Days button updates timeDelta", async () => {
  const timeDeltaStore = writable(TIME_DELTA_1_YEAR);

  render(ChartTimeFiltersTest, { timeDeltaStore });

  const thirtyDaysButton = screen.getByText("30 Days");
  await fireEvent.click(thirtyDaysButton);

  expect(thirtyDaysButton).toBeDisabled();
  expect(get(timeDeltaStore)).toBe(TIME_DELTA_30_DAYS);
});

test("clicking 7 Days button updates timeDelta", async () => {
  const timeDeltaStore = writable(TIME_DELTA_1_YEAR);

  render(ChartTimeFiltersTest, { timeDeltaStore });

  const sevenDaysButton = screen.getByText("7 Days");
  await fireEvent.click(sevenDaysButton);

  expect(sevenDaysButton).toBeDisabled();
  expect(get(timeDeltaStore)).toBe(TIME_DELTA_7_DAYS);
});

test("clicking 24 Hours button updates timeDelta", async () => {
  const timeDeltaStore = writable(TIME_DELTA_1_YEAR);

  render(ChartTimeFiltersTest, { timeDeltaStore });

  const twentyFourHoursButton = screen.getByText("24 Hours");
  await fireEvent.click(twentyFourHoursButton);

  expect(twentyFourHoursButton).toBeDisabled();
  expect(get(timeDeltaStore)).toBe(TIME_DELTA_24_HOURS);
});

test("clicking 1 Year button updates timeDelta", async () => {
  const timeDeltaStore = writable(TIME_DELTA_30_DAYS);

  render(ChartTimeFiltersTest, { timeDeltaStore });

  const yearButton = screen.getByText("1 Year");
  await fireEvent.click(yearButton);

  expect(yearButton).toBeDisabled();
  expect(get(timeDeltaStore)).toBe(TIME_DELTA_1_YEAR);
});
