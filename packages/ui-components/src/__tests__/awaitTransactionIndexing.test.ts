import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import type { RaindexOrder, RaindexTransaction } from '@rainlanguage/raindex';
import { awaitSubgraphIndexing } from '$lib/services/awaitTransactionIndexing';
import { TransactionStoreErrorMessage } from '$lib/types/transaction';

describe('subgraphIndexing', () => {
	const mockFetchData = vi.fn();

	beforeEach(() => {
		vi.resetAllMocks();
		vi.useFakeTimers();
	});

	afterEach(() => {
		vi.useRealTimers();
		vi.clearAllMocks();
	});

	it('should resolve with value when data is successfully fetched', async () => {
		const mockData = { id: '0x0123' } as unknown as RaindexTransaction;
		mockFetchData.mockResolvedValue({ value: mockData });

		const resultPromise = awaitSubgraphIndexing({
			chainId: 1,
			raindex: '0x123',
			txHash: '0x0234',
			successMessage: 'Transaction confirmed',
			fetchEntityFn: mockFetchData,
			isSuccess: (data: RaindexTransaction) => !!data.id
		});

		await vi.advanceTimersByTimeAsync(1000);

		const result = await resultPromise;

		expect(result.value).toBeDefined();
		expect(result.error).toBeUndefined();
		expect(result.value?.txHash).toBe('0x0234');
		expect(result.value?.successMessage).toBe('Transaction confirmed');
		expect(result.value?.data).toEqual(mockData);

		expect(mockFetchData).toHaveBeenCalledWith(1, '0x123', '0x0234');
		expect(mockFetchData).toHaveBeenCalledTimes(1);
	});

	it('should extract order hash from array data', async () => {
		const mockOrderData = [
			{
				orderHash: 'order123'
			}
		] as unknown as RaindexOrder[];

		mockFetchData.mockResolvedValue({ value: mockOrderData });

		const resultPromise = awaitSubgraphIndexing({
			chainId: 1,
			raindex: '0x123',
			txHash: '0x0234',
			successMessage: 'Order confirmed',
			fetchEntityFn: mockFetchData,
			isSuccess: (data: RaindexOrder[]) => data.length > 0
		});

		await vi.advanceTimersByTimeAsync(1000);

		const result = await resultPromise;

		expect(result.value).toBeDefined();
		expect(result.value?.orderHash).toBe('order123');
	});

	it('should retry fetching data until maxAttempts is reached', async () => {
		mockFetchData.mockResolvedValue({ value: null });

		const resultPromise = awaitSubgraphIndexing({
			chainId: 1,
			raindex: '0x123',
			txHash: '0x0234',
			successMessage: 'Transaction confirmed',
			maxAttempts: 5,
			interval: 500,
			fetchEntityFn: mockFetchData,
			isSuccess: () => false
		});

		for (let i = 0; i < 5; i++) {
			await vi.advanceTimersByTimeAsync(500);
		}

		const result = await resultPromise;

		expect(result.error).toBeDefined();
		expect(result.value).toBeUndefined();
		expect(result.error).toBe(TransactionStoreErrorMessage.SUBGRAPH_TIMEOUT_ERROR);
		expect(mockFetchData).toHaveBeenCalledTimes(5);
	});

	it('should handle fetch errors gracefully', async () => {
		mockFetchData.mockResolvedValue({ error: 'error' });

		const resultPromise = awaitSubgraphIndexing({
			chainId: 1,
			raindex: '0x123',
			txHash: '0x0234',
			successMessage: 'Transaction confirmed',
			maxAttempts: 3,
			interval: 500,
			fetchEntityFn: mockFetchData,
			isSuccess: () => true
		});

		for (let i = 0; i < 3; i++) {
			await vi.advanceTimersByTimeAsync(500);
		}

		const result = await resultPromise;

		expect(result.error).toBeDefined();
		expect(result.value).toBeUndefined();
		expect(result.error).toBe(TransactionStoreErrorMessage.SUBGRAPH_TIMEOUT_ERROR);
	});

	it('should resolve immediately when successful data is found', async () => {
		mockFetchData
			.mockResolvedValueOnce({ value: null })
			.mockResolvedValueOnce({ value: { id: '0x0123' } as unknown as RaindexTransaction });

		const resultPromise = awaitSubgraphIndexing({
			chainId: 1,
			raindex: '0x123',
			txHash: '0x0234',
			successMessage: 'Transaction confirmed',
			maxAttempts: 5,
			interval: 500,
			fetchEntityFn: mockFetchData,
			isSuccess: (data: RaindexTransaction) => !!data?.id
		});

		await vi.advanceTimersByTimeAsync(500);
		await vi.advanceTimersByTimeAsync(500);

		const result = await resultPromise;

		expect(result.value).toBeDefined();
		expect(result.error).toBeUndefined();
		expect(mockFetchData).toHaveBeenCalledTimes(2);
	});
});
