import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, vi, type Mock } from 'vitest';
import { QueryClient } from '@tanstack/svelte-query';
import TanstackOrderQuote from '../lib/components/detail/TanstackOrderQuote.svelte';
import { expect } from '../lib/test/matchers';
import type { RaindexOrder } from '@rainlanguage/raindex';
import { useToasts } from '$lib/providers/toasts/useToasts';
import { writable } from 'svelte/store';
import { invalidateTanstackQueries } from '$lib/queries/queryClient';

vi.mock('@rainlanguage/raindex', () => ({
	getOrderQuote: vi.fn()
}));

vi.mock('$lib/providers/toasts/useToasts', () => ({
	useToasts: vi.fn()
}));

vi.mock('$lib/queries/queryClient', async (importOriginal) => ({
	...((await importOriginal()) as object),
	invalidateTanstackQueries: vi.fn()
}));

const mockErrToast = vi.fn();
const mockAddToast = vi.fn();
let clipboardWriteText: ReturnType<typeof vi.fn>;

describe('TanstackOrderQuote component', () => {
	const mockOrder: RaindexOrder = {
		id: '1',
		getQuotes: vi.fn()
	} as unknown as RaindexOrder;

	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetAllMocks();
		mockErrToast.mockReset();
		mockAddToast.mockReset();
		clipboardWriteText = vi.fn().mockResolvedValue(undefined);
		Object.defineProperty(window.navigator, 'clipboard', {
			value: { writeText: clipboardWriteText },
			configurable: true
		});
		(useToasts as Mock).mockReturnValue({
			toasts: writable([]),
			errToast: mockErrToast,
			addToast: mockAddToast,
			removeToast: vi.fn()
		});
	});
	it('displays order quote data when query is successful', async () => {
		(mockOrder.getQuotes as Mock).mockResolvedValueOnce({
			value: [
				{
					success: true,
					block_number: '0x123',
					pair: { pairName: 'ETH/USDT', inputIndex: 0, outputIndex: 1 },
					data: {
						formattedMaxOutput: '1.550122181502135692',
						formattedRatio: '6.563567234157974775'
					},
					error: undefined
				}
			]
		});

		const queryClient = new QueryClient();

		render(TanstackOrderQuote, {
			props: {
				order: mockOrder,
				handleQuoteDebugModal: vi.fn()
			},
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			const orderQuoteComponent = screen.getByTestId('bodyRow');

			expect(orderQuoteComponent).toHaveTextContent('ETH/USDT');
			expect(orderQuoteComponent).toHaveTextContent('1.550122181502135692');
			expect(orderQuoteComponent).toHaveTextContent('6.563567234157974775');
		});
	});

	it('refreshes the quote when the refresh icon is clicked', async () => {
		(mockOrder.getQuotes as Mock).mockResolvedValueOnce({
			value: [
				{
					success: true,
					block_number: '0x123',
					pair: { pairName: 'ETH/USDT', inputIndex: 0, outputIndex: 1 },
					data: { formattedMaxOutput: '1.550122181502135692' },
					error: undefined
				},
				{
					success: true,
					block_number: '0x123',
					pair: { pairName: 'BTC/USDT', inputIndex: 0, outputIndex: 1 },
					data: { formattedMaxOutput: '6.123350635480882605' },
					error: undefined
				}
			]
		});
		// Setup mock for the second data load (after refresh)
		(mockOrder.getQuotes as Mock).mockResolvedValueOnce({
			value: [
				{
					success: true,
					block_number: '0x123',
					pair: { pairName: 'ETH/USDT', inputIndex: 0, outputIndex: 1 },
					data: {
						formattedMaxOutput: '5.945438972656012126',
						formattedRatio: '6.305004957644166012'
					},
					error: undefined
				},
				{
					success: true,
					block_number: '0x123',
					pair: { pairName: 'BTC/USDT', inputIndex: 0, outputIndex: 1 },
					data: {
						formattedMaxOutput: '6.066479884955967059',
						formattedRatio: '6.485589855485802559'
					},
					error: undefined
				}
			]
		});

		// Mock the invalidateTanstackQueries function to ensure it works correctly
		(invalidateTanstackQueries as Mock).mockImplementation((queryClient, queryKey) => {
			// This will trigger a refetch which will use the second mock
			queryClient.invalidateQueries({ queryKey });
			return Promise.resolve();
		});

		const queryClient = new QueryClient();

		render(TanstackOrderQuote, {
			props: {
				order: mockOrder,
				handleQuoteDebugModal: vi.fn()
			},
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			const orderQuoteRows = screen.getAllByTestId('bodyRow');

			// Check ETH/USDT row
			expect(orderQuoteRows[0]).toHaveTextContent('1.550122181502135692');

			// Check BTC/USDT row
			expect(orderQuoteRows[1]).toHaveTextContent('6.123350635480882605');
		});

		const refreshButton = screen.getByTestId('refresh-button');
		fireEvent.click(refreshButton);

		await waitFor(() => {
			const orderQuoteRows = screen.getAllByTestId('bodyRow');

			// Check ETH/USD row
			expect(orderQuoteRows[0]).toHaveTextContent('ETH/USD');
			expect(orderQuoteRows[0]).toHaveTextContent('5.945438972656012126');
			expect(orderQuoteRows[0]).toHaveTextContent('6.305004957644166012');

			// Check BTC/USDT row
			expect(orderQuoteRows[1]).toHaveTextContent('BTC/USDT');
			expect(orderQuoteRows[1]).toHaveTextContent('6.066479884955967059');
			expect(orderQuoteRows[1]).toHaveTextContent('6.485589855485802559');
		});
	});

	it('displays error message when query fails', async () => {
		(mockOrder.getQuotes as Mock).mockResolvedValueOnce({
			value: [
				{
					success: false,
					block_number: '0x123',
					pair: { pairName: 'ETH/USDT', inputIndex: 0, outputIndex: 1 },
					data: undefined,
					error: 'Network error'
				}
			]
		});

		const queryClient = new QueryClient();

		render(TanstackOrderQuote, {
			props: {
				order: mockOrder,
				handleQuoteDebugModal: vi.fn()
			},
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			const errorNodes = screen.queryAllByText(/Network error/);
			expect(errorNodes.length).toBeGreaterThan(0);
		});
	});

	it('displays zero for price when io ratio is zero', async () => {
		(mockOrder.getQuotes as Mock).mockResolvedValueOnce({
			value: [
				{
					success: true,
					block_number: '0x123',
					pair: { pairName: 'ETH/USDT', inputIndex: 0, outputIndex: 1 },
					data: {
						formattedMaxOutput: '1.550122181502135692',
						formattedRatio: '0',
						formattedInverseRatio: '0'
					},
					error: undefined
				}
			]
		});

		const queryClient = new QueryClient();

		render(TanstackOrderQuote, {
			props: {
				order: mockOrder,
				handleQuoteDebugModal: vi.fn()
			},
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			const orderQuoteComponent = screen.getByTestId('bodyRow');

			expect(orderQuoteComponent).toHaveTextContent('ETH/USDT');
			expect(orderQuoteComponent).toHaveTextContent('1.550122181502135692'); // maxOutput
			expect(orderQuoteComponent).toHaveTextContent('0'); // ratio
			expect(orderQuoteComponent).toHaveTextContent('(0)'); // inverse price
		});
	});

	it('triggers a toast when the quote fails to refresh', async () => {
		(invalidateTanstackQueries as Mock).mockRejectedValueOnce(new Error('Failed to refresh'));
		(mockOrder.getQuotes as Mock).mockResolvedValueOnce({
			value: [
				{
					success: true,
					block_number: '0x123',
					pair: { pairName: 'ETH/USDT', inputIndex: 0, outputIndex: 1 },
					data: { maxOutput: '0x158323e942e36d8c', ratio: '0x5b16799fcb6114f7' },
					error: undefined
				},
				{
					success: true,
					block_number: '0x123',
					pair: { pairName: 'BTC/USDT', inputIndex: 0, outputIndex: 1 },
					data: { maxOutput: '0x54fa82f5c7001dad', ratio: '0x53e0089714d06709' },
					error: undefined
				}
			]
		});
		const queryClient = new QueryClient();

		render(TanstackOrderQuote, {
			props: {
				order: mockOrder,
				handleQuoteDebugModal: vi.fn()
			},
			context: new Map([['$$_queryClient', queryClient]])
		});

		const refreshButton = screen.getByTestId('refresh-button');
		fireEvent.click(refreshButton);

		await waitFor(() => {
			expect(mockErrToast).toHaveBeenCalledWith('Failed to refresh');
		});
	});

	it('copies the quote error to the clipboard and notifies the user', async () => {
		(mockOrder.getQuotes as Mock).mockResolvedValueOnce({
			value: [
				{
					success: false,
					block_number: '0x123',
					pair: { pairName: 'ETH/USDT', inputIndex: 0, outputIndex: 1 },
					data: undefined,
					error: 'Failed to calculate quote'
				}
			]
		});

		const queryClient = new QueryClient();

		render(TanstackOrderQuote, {
			props: {
				order: mockOrder,
				handleQuoteDebugModal: vi.fn()
			},
			context: new Map([['$$_queryClient', queryClient]])
		});

		const copyButton = await screen.findByRole('button', { name: 'Copy quote error' });
		await fireEvent.click(copyButton);

		await waitFor(() => {
			expect(clipboardWriteText).toHaveBeenCalledWith('Failed to calculate quote');
			expect(mockAddToast).toHaveBeenCalledWith({
				message: 'Copied quote error',
				type: 'success',
				color: 'green'
			});
			expect(mockErrToast).not.toHaveBeenCalled();
		});
	});

	it('displays truncated amounts with tooltips', async () => {
		(mockOrder.getQuotes as Mock).mockResolvedValueOnce({
			value: [
				{
					success: true,
					blockNumber: '0x123',
					pair: { pairName: 'ETH/USDT', inputIndex: 0, outputIndex: 1 },
					data: {
						formattedMaxOutput: '1.550122181502135692',
						formattedRatio: '6.563567234157974775',
						formattedInverseRatio: '0.152345678901234567',
						formattedMaxInput: '10.175432109876543210'
					},
					error: undefined
				}
			]
		});

		const queryClient = new QueryClient();

		render(TanstackOrderQuote, {
			props: {
				order: mockOrder,
				handleQuoteDebugModal: vi.fn()
			},
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			const maxOutputSpan = document.getElementById('max-output-0');
			expect(maxOutputSpan).not.toBeNull();
			expect(maxOutputSpan?.classList.contains('truncate')).toBe(true);
			expect(maxOutputSpan).toHaveTextContent('1.550122181502135692');

			const ratioSpan = document.getElementById('ratio-0');
			expect(ratioSpan).not.toBeNull();
			expect(ratioSpan?.classList.contains('truncate')).toBe(true);
			expect(ratioSpan).toHaveTextContent('6.563567234157974775');
			expect(ratioSpan).toHaveTextContent('0.152345678901234567');

			const maxInputSpan = document.getElementById('max-input-0');
			expect(maxInputSpan).not.toBeNull();
			expect(maxInputSpan?.classList.contains('truncate')).toBe(true);
			expect(maxInputSpan).toHaveTextContent('10.175432109876543210');
		});
	});
});
