import { render, fireEvent, screen } from "@testing-library/svelte";
import { get, writable } from "svelte/store";
import { test, expect } from "vitest";
import TableTimeFiltersTest from "./TableTimeFilters.test.svelte";

const TIME_DELTA_24_HOURS = 60 * 60 * 24;
const TIME_DELTA_48_HOURS = TIME_DELTA_24_HOURS * 2;

test("initial start/end time difference is set to all time", async () => {
  const startTimeStore = writable<number | undefined>();
  const endTimeStore = writable<number | undefined>();

  render(TableTimeFiltersTest, { startTimeStore, endTimeStore });

  const twentyFourHoursButton = screen.getByText("24 Hours");
  expect(twentyFourHoursButton).toBeEnabled();
  expect(get(endTimeStore)).toBe(undefined);
  expect(get(startTimeStore)).toBe(undefined);
});

test("clicking All Time button updates timeDelta", async () => {
  const startTimeStore = writable(0);
  const endTimeStore = writable(0);

  render(TableTimeFiltersTest, { startTimeStore, endTimeStore });

  const allTimeButton = screen.getByText("All Time");
  await fireEvent.click(allTimeButton);

  expect(allTimeButton).toBeDisabled();
  expect(get(startTimeStore)).toBe(undefined);
  expect(get(endTimeStore)).toBe(undefined);
});

test("clicking 48 Hours button updates start/end timestamp", async () => {
  const startTimeStore = writable(0);
  const endTimeStore = writable(0);

  render(TableTimeFiltersTest, { startTimeStore, endTimeStore });

  const fortyEightHoursButton = screen.getByText("48 Hours");
  await fireEvent.click(fortyEightHoursButton);

  expect(fortyEightHoursButton).toBeDisabled();
  expect(get(endTimeStore) - get(startTimeStore)).toBe(TIME_DELTA_48_HOURS);
});

test("clicking 24 Hours button updates start/end timestamp", async () => {
  const startTimeStore = writable(0);
  const endTimeStore = writable(0);

  render(TableTimeFiltersTest, { startTimeStore, endTimeStore });

  const twentyFourHoursButton = screen.getByText("24 Hours");
  await fireEvent.click(twentyFourHoursButton);

  expect(twentyFourHoursButton).toBeDisabled();
  expect(get(endTimeStore) - get(startTimeStore)).toBe(TIME_DELTA_24_HOURS);
});
