import { act, fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '../lib/test/matchers';
import LightweightChart from '../lib/components/charts/LightweightChart.svelte';
import { type IChartApi, type UTCTimestamp } from 'lightweight-charts';
import { readable } from 'svelte/store';
const setDataMock = vi.fn();
const applyOptionsMock = vi.fn();
const setVisibleRangeMock = vi.fn();
const removeMock = vi.fn();
const updateMock = vi.fn();

vi.mock('lightweight-charts', async (importOriginal) => ({
	...((await importOriginal()) as object),
	createChart: vi.fn(() => ({
		addLineSeries: vi.fn(() => ({
			setData: setDataMock,
			update: updateMock
		})),
		remove: removeMock,
		applyOptions: applyOptionsMock,
		timeScale: vi.fn(() => ({
			setVisibleRange: setVisibleRangeMock
		}))
	}))
}));

test('renders without data correctly', async () => {
	const title = 'test title';
	const emptyMessage = 'empty message';
	let loading = true;
	const priceSymbol = '$';
	const createSeries = (chart: IChartApi) => chart.addLineSeries();

	const { component } = render(LightweightChart, {
		title,
		emptyMessage,
		loading,
		priceSymbol,
		createSeries,
		lightweightChartsTheme: readable({ test: 'test' })
	});

	await waitFor(() => {
		expect(screen.getByTestId('lightweightChartTitle')).toHaveTextContent(title);
		expect(screen.getByTestId('lightweightChartSpinner')).toBeInTheDocument();
	});

	loading = false;

	await act(() => component.$set({ loading: false }));

	await waitFor(() => {
		expect(screen.getByTestId('lightweightChartEmptyMessage')).toHaveTextContent(emptyMessage);
	});
});

test('renders with data correctly', async () => {
	const title = 'test title';
	const emptyMessage = 'empty message';
	const loading = true;
	const priceSymbol = '$';

	const createSeries = (chart: IChartApi) => chart.addLineSeries();

	const data: { value: number; time: UTCTimestamp; color?: string }[] = [
		{
			value: 10,
			time: 1529899200 as UTCTimestamp
		},
		{
			value: 20,
			time: 1529899300 as UTCTimestamp
		},
		{
			value: 5,
			time: 1529899500 as UTCTimestamp
		}
	];

	render(LightweightChart, {
		title,
		emptyMessage,
		loading,
		priceSymbol,
		createSeries,
		data,
		lightweightChartsTheme: readable({ test: 'test' })
	});

	await waitFor(() => {
		expect(screen.getByTestId('lightweightChartTitle')).toHaveTextContent(title);
		expect(screen.getByTestId('lightweightChartSpinner')).toBeInTheDocument();
		expect(screen.getByTestId('lightweightChartElement')).toBeInTheDocument();
		expect(screen.queryByTestId('lightweightChartYearButtons')).toBeInTheDocument();

		expect(setDataMock).toHaveBeenCalledWith(data);
	});
});

test('updates data correctly when new data points are added', async () => {
	const title = 'test title';
	const emptyMessage = 'empty message';
	const loading = false;
	const priceSymbol = '$';
	const createSeries = (chart: IChartApi) => chart.addLineSeries();

	const initialData = [
		{ value: 10, time: 1529899200 as UTCTimestamp },
		{ value: 20, time: 1529899300 as UTCTimestamp }
	];

	const { component } = render(LightweightChart, {
		title,
		emptyMessage,
		loading,
		priceSymbol,
		createSeries,
		data: initialData,
		lightweightChartsTheme: readable({ test: 'test' })
	});

	// First render should call setData with initial data
	await waitFor(() => {
		expect(setDataMock).toHaveBeenCalledWith(initialData);
	});

	// Add new data points
	const newDataPoints = [
		...initialData,
		{ value: 30, time: 1529899400 as UTCTimestamp },
		{ value: 40, time: 1529899500 as UTCTimestamp }
	];

	// Update with new data that includes additional points
	await act(() => component.$set({ data: newDataPoints }));

	// Should call update for each new point
	await waitFor(() => {
		expect(updateMock).toHaveBeenCalledTimes(2);
		expect(updateMock).toHaveBeenCalledWith({
			value: 30,
			time: 1529899400 as UTCTimestamp
		});
		expect(updateMock).toHaveBeenCalledWith({
			value: 40,
			time: 1529899500 as UTCTimestamp
		});
	});
});

test('setOptions is called correctly', async () => {
	const title = 'test title';
	const emptyMessage = 'empty message';
	const loading = false;
	const priceSymbol = '$';

	render(LightweightChart, {
		title,
		emptyMessage,
		loading,
		priceSymbol,
		createSeries: (chart: IChartApi) => chart.addLineSeries(),
		lightweightChartsTheme: readable({ test: 'test' })
	});

	await waitFor(() => {
		expect(applyOptionsMock).toHaveBeenCalled();

		const callArgs = applyOptionsMock.mock.calls[0][0];
		expect(callArgs).toMatchObject({
			test: 'test'
		});
	});
});

test('setTimeScale is called correctly', async () => {
	const fakeNow = new Date('2025-06-15T12:00:00Z');
	vi.useFakeTimers({ shouldAdvanceTime: true });
	vi.setSystemTime(fakeNow);

	const title = 'test title';
	const emptyMessage = 'empty message';
	const loading = false;
	const priceSymbol = '$';

	const data: { value: number; time: UTCTimestamp; color?: string }[] = [
		{
			value: 10,
			time: 1529899200 as UTCTimestamp
		},
		{
			value: 20,
			time: 1529899300 as UTCTimestamp
		},
		{
			value: 5,
			time: 1529899500 as UTCTimestamp
		}
	];

	render(LightweightChart, {
		title,
		emptyMessage,
		loading,
		priceSymbol,
		createSeries: (chart: IChartApi) => chart.addLineSeries(),
		data,
		lightweightChartsTheme: readable({ test: 'test' })
	});

	// Simulate clicking the "30 Days" button to change the timeDelta
	await waitFor(async () => {
		const timeDeltaButton = screen.getByText('30 Days');
		await act(() => fireEvent.click(timeDeltaButton));
	});

	const timeDelta = 60 * 60 * 24 * 30; // 30 days in seconds
	const timeTo = Math.floor(fakeNow.getTime() / 1000) as UTCTimestamp;
	const timeFrom = (timeTo - timeDelta) as UTCTimestamp;

	await waitFor(() => {
		expect(setVisibleRangeMock).toHaveBeenCalledWith({
			from: timeFrom,
			to: timeTo
		});
	});

	vi.useRealTimers();
});

test('setupChart is called correctly', async () => {
	const title = 'test title';
	const emptyMessage = 'empty message';
	const loading = false;
	const priceSymbol = '$';

	const data: { value: number; time: UTCTimestamp; color?: string }[] = [
		{
			value: 10,
			time: 1529899200 as UTCTimestamp
		},
		{
			value: 20,
			time: 1529899300 as UTCTimestamp
		},
		{
			value: 5,
			time: 1529899500 as UTCTimestamp
		}
	];

	render(LightweightChart, {
		title,
		emptyMessage,
		loading,
		priceSymbol,
		createSeries: (chart: IChartApi) => chart.addLineSeries(),
		data,
		lightweightChartsTheme: readable({ test: 'test' })
	});

	await waitFor(() => {
		expect(screen.getByTestId('lightweightChartElement')).toBeInTheDocument();
		expect(setDataMock).toHaveBeenCalledWith(data);
	});
});

test('destroyChart is called correctly', async () => {
	const title = 'test title';
	const emptyMessage = 'empty message';
	const loading = false;
	const priceSymbol = '$';

	const { component } = render(LightweightChart, {
		title,
		emptyMessage,
		loading,
		priceSymbol,
		createSeries: (chart: IChartApi) => chart.addLineSeries(),
		lightweightChartsTheme: readable({ test: 'test' })
	});

	component.$destroy();

	await waitFor(() => {
		expect(removeMock).toHaveBeenCalled();
	});
});
