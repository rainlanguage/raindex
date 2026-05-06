import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi, type Mock } from 'vitest';
import { expect } from '../lib/test/matchers';
import type { RaindexOrder, RaindexTrade } from '@rainlanguage/orderbook';
import OrderTradesListTable from '../lib/components/tables/OrderTradesListTable.svelte';
import { QueryClient } from '@tanstack/svelte-query';

const mockTradeOrdersList: RaindexTrade[] = [
	{
		id: '1',
		timestamp: BigInt(1632000000),
		transaction: {
			id: 'transaction_id_1',
			from: '0xsender_address',
			timestamp: BigInt(1632000000),
			blockNumber: BigInt(0)
		},
		outputVaultBalanceChange: {
			amount: BigInt(-100),
			formattedAmount: '-100',
			vaultId: BigInt(1),
			token: {
				id: 'output_token',
				address: '0xoutput_token',
				name: 'Output Token',
				symbol: 'OUT',
				decimals: '1'
			},
			id: '1',
			__typename: 'Withdraw',
			newBalance: BigInt(0),
			formattedNewBalance: '0',
			oldBalance: BigInt(0),
			formattedOldBalance: '0',
			timestamp: BigInt(0),
			transaction: {
				id: 'transaction_id_1',
				from: '0xsender_address',
				timestamp: BigInt(1632000000),
				blockNumber: BigInt(0)
			},
			orderbook: '0x1'
		},
		orderHash: 'orderHash',
		inputVaultBalanceChange: {
			vaultId: BigInt(1),
			token: {
				id: 'input_token',
				address: '0xinput_token',
				name: 'Input Token',
				symbol: 'INP',
				decimals: '1'
			},
			amount: BigInt(50),
			formattedAmount: '50',
			id: '1',
			__typename: 'Withdraw',
			newBalance: BigInt(0),
			formattedNewBalance: '0',
			oldBalance: BigInt(0),
			formattedOldBalance: '0',
			timestamp: BigInt(0),
			transaction: {
				id: 'transaction_id_1',
				from: '0xsender_address',
				timestamp: BigInt(1632000000),
				blockNumber: BigInt(0)
			},
			orderbook: '0x1'
		},
		orderbook: '0x00',
		tradeEventType: 'TakeOrder',
		counterparty: {
			owner: '0xsender_address',
			orderHash: undefined
		}
	},
	{
		id: '2',
		timestamp: BigInt(1632000000),
		transaction: {
			id: 'transaction_id_2',
			from: '0xsender_address',
			timestamp: BigInt(1632000000),
			blockNumber: BigInt(0)
		},
		outputVaultBalanceChange: {
			amount: BigInt(-100),
			formattedAmount: '-100',
			vaultId: BigInt(1),
			token: {
				id: 'output_token',
				address: '0xoutput_token',
				name: 'Output Token',
				symbol: 'OUT',
				decimals: '1'
			},
			id: '1',
			__typename: 'Withdraw',
			newBalance: BigInt(0),
			formattedNewBalance: '0',
			oldBalance: BigInt(0),
			formattedOldBalance: '0',
			timestamp: BigInt(0),
			transaction: {
				id: 'transaction_id_2',
				from: '0xsender_address',
				timestamp: BigInt(1632000000),
				blockNumber: BigInt(0)
			},
			orderbook: '0x1'
		},
		orderHash: 'orderHash',
		inputVaultBalanceChange: {
			vaultId: BigInt(1),
			token: {
				id: 'input_token',
				address: '0xinput_token',
				name: 'Input Token',
				symbol: 'INP',
				decimals: '1'
			},
			amount: BigInt(50),
			formattedAmount: '50',
			id: '1',
			__typename: 'Withdraw',
			newBalance: BigInt(0),
			formattedNewBalance: '0',
			oldBalance: BigInt(0),
			formattedOldBalance: '0',
			timestamp: BigInt(0),
			transaction: {
				id: 'transaction_id_2',
				from: '0xsender_address',
				timestamp: BigInt(1632000000),
				blockNumber: BigInt(0)
			},
			orderbook: '0x1'
		},
		orderbook: '0x00',
		tradeEventType: 'TakeOrder',
		counterparty: {
			owner: '0xsender_address',
			orderHash: undefined
		}
	}
] as unknown as RaindexTrade[];

vi.mock('@tanstack/svelte-query');

const mockOrder: RaindexOrder = {
	id: '1',
	chainId: BigInt(1),
	orderbook: '0x00',
	getTradeCount: vi.fn(),
	getTradesList: vi.fn()
} as unknown as RaindexOrder;

test('renders table with correct data', async () => {
	const queryClient = new QueryClient();

	const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		subscribe: (fn: (value: any) => void) => {
			fn({
				data: { pages: [mockTradeOrdersList] },
				status: 'success',
				isFetching: false,
				isFetched: true
			});
			return { unsubscribe: () => {} };
		}
	})) as Mock;

	render(OrderTradesListTable, {
		context: new Map([['$$_queryClient', queryClient]]),
		props: { order: mockOrder, rpcs: ['https://example.com'] }
	});

	await waitFor(async () => {
		// get all the io ratios
		const rows = screen.getAllByTestId('io-ratio');

		// checking the io ratios
		for (let i = 0; i < mockTradeOrdersList.length; i++) {
			const inputDisplay = mockTradeOrdersList[i].inputVaultBalanceChange.formattedAmount;
			const outputDisplay = mockTradeOrdersList[i].outputVaultBalanceChange.formattedAmount;
			const ioRatio = Number(inputDisplay) / (Number(outputDisplay) * -1);
			const oiRatio = (Number(outputDisplay) * -1) / Number(inputDisplay);
			expect(rows[i]).toHaveTextContent(ioRatio.toString());
			expect(rows[i]).toHaveTextContent(oiRatio.toString());
		}
	});
});

test('renders a debug button for each trade', async () => {
	const queryClient = new QueryClient();

	const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		subscribe: (fn: (value: any) => void) => {
			fn({
				data: { pages: [mockTradeOrdersList] },
				status: 'success',
				isFetching: false,
				isFetched: true
			});
			return { unsubscribe: () => {} };
		}
	})) as Mock;

	render(OrderTradesListTable, {
		context: new Map([['$$_queryClient', queryClient]]),
		props: {
			order: mockOrder,
			rpcs: ['example.com'],
			handleDebugTradeModal: () => {}
		}
	});

	await waitFor(async () => {
		const buttons = screen.getAllByTestId('debug-trade-button');
		expect(buttons).toHaveLength(mockTradeOrdersList.length);
	});
});

test('renders combined Transaction column with Sender and Tx', async () => {
	const queryClient = new QueryClient();

	const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		subscribe: (fn: (value: any) => void) => {
			fn({
				data: { pages: [mockTradeOrdersList] },
				status: 'success',
				isFetching: false,
				isFetched: true
			});
			return { unsubscribe: () => {} };
		}
	})) as Mock;

	render(OrderTradesListTable, {
		context: new Map([['$$_queryClient', queryClient]]),
		props: { order: mockOrder, rpcs: ['https://example.com'] }
	});

	await waitFor(async () => {
		const rows = screen.getAllByTestId('bodyRow');
		expect(rows).toHaveLength(mockTradeOrdersList.length);

		// Each row should have the Transaction column with Sender and Tx labels
		rows.forEach((row) => {
			expect(row).toHaveTextContent('Sender:');
			expect(row).toHaveTextContent('Tx:');
		});
	});
});

test('renders Input column with token symbol and amount', async () => {
	const queryClient = new QueryClient();

	const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		subscribe: (fn: (value: any) => void) => {
			fn({
				data: { pages: [mockTradeOrdersList] },
				status: 'success',
				isFetching: false,
				isFetched: true
			});
			return { unsubscribe: () => {} };
		}
	})) as Mock;

	render(OrderTradesListTable, {
		context: new Map([['$$_queryClient', queryClient]]),
		props: { order: mockOrder, rpcs: ['https://example.com'] }
	});

	await waitFor(async () => {
		const inputCells = screen.getAllByTestId('input');
		expect(inputCells).toHaveLength(mockTradeOrdersList.length);

		// Each input cell should show token symbol and amount
		inputCells.forEach((cell, i) => {
			expect(cell).toHaveTextContent(mockTradeOrdersList[i].inputVaultBalanceChange.token.symbol!);
			expect(cell).toHaveTextContent(
				mockTradeOrdersList[i].inputVaultBalanceChange.formattedAmount
			);
		});
	});
});

test('renders Output column with token symbol and amount', async () => {
	const queryClient = new QueryClient();

	const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		subscribe: (fn: (value: any) => void) => {
			fn({
				data: { pages: [mockTradeOrdersList] },
				status: 'success',
				isFetching: false,
				isFetched: true
			});
			return { unsubscribe: () => {} };
		}
	})) as Mock;

	render(OrderTradesListTable, {
		context: new Map([['$$_queryClient', queryClient]]),
		props: { order: mockOrder, rpcs: ['https://example.com'] }
	});

	await waitFor(async () => {
		const outputCells = screen.getAllByTestId('output');
		expect(outputCells).toHaveLength(mockTradeOrdersList.length);

		// Each output cell should show token symbol and amount
		outputCells.forEach((cell, i) => {
			expect(cell).toHaveTextContent(mockTradeOrdersList[i].outputVaultBalanceChange.token.symbol!);
			expect(cell).toHaveTextContent(
				mockTradeOrdersList[i].outputVaultBalanceChange.formattedAmount
			);
		});
	});
});

const createMockTrade = (id: string, inputAmount: string, outputAmount: string): RaindexTrade =>
	({
		id,
		timestamp: BigInt(1632000000),
		transaction: {
			id: `tx_${id}`,
			from: '0xsender_address',
			timestamp: BigInt(1632000000),
			blockNumber: BigInt(0)
		},
		outputVaultBalanceChange: {
			amount: BigInt(-100),
			formattedAmount: outputAmount,
			vaultId: BigInt(1),
			token: {
				id: 'output_token',
				address: '0xoutput_token',
				name: 'Output Token',
				symbol: 'OUT',
				decimals: '1'
			},
			id: '1',
			__typename: 'Withdraw',
			newBalance: BigInt(0),
			formattedNewBalance: '0',
			oldBalance: BigInt(0),
			formattedOldBalance: '0',
			timestamp: BigInt(0),
			transaction: {
				id: `tx_${id}`,
				from: '0xsender_address',
				timestamp: BigInt(1632000000),
				blockNumber: BigInt(0)
			},
			orderbook: '0x1'
		},
		orderHash: 'orderHash',
		inputVaultBalanceChange: {
			vaultId: BigInt(1),
			token: {
				id: 'input_token',
				address: '0xinput_token',
				name: 'Input Token',
				symbol: 'INP',
				decimals: '1'
			},
			amount: BigInt(50),
			formattedAmount: inputAmount,
			id: '1',
			__typename: 'Withdraw',
			newBalance: BigInt(0),
			formattedNewBalance: '0',
			oldBalance: BigInt(0),
			formattedOldBalance: '0',
			timestamp: BigInt(0),
			transaction: {
				id: `tx_${id}`,
				from: '0xsender_address',
				timestamp: BigInt(1632000000),
				blockNumber: BigInt(0)
			},
			orderbook: '0x1'
		},
		orderbook: '0x00',
		tradeEventType: 'TakeOrder',
		counterparty: {
			owner: '0xsender_address',
			orderHash: undefined
		}
	}) as unknown as RaindexTrade;

test('displays dash when output amount is zero (prevents division by zero)', async () => {
	const queryClient = new QueryClient();
	const mockTrades = [createMockTrade('1', '50', '0')];

	const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
		subscribe: (fn: (value: unknown) => void) => {
			fn({
				data: { pages: [mockTrades] },
				status: 'success',
				isFetching: false,
				isFetched: true
			});
			return { unsubscribe: () => {} };
		}
	})) as Mock;

	render(OrderTradesListTable, {
		context: new Map([['$$_queryClient', queryClient]]),
		props: { order: mockOrder, rpcs: ['https://example.com'] }
	});

	await waitFor(() => {
		const ioRatioCell = screen.getByTestId('io-ratio');
		expect(ioRatioCell).toHaveTextContent('-');
		expect(ioRatioCell).not.toHaveTextContent('Infinity');
	});
});

test('displays dash when input amount is zero', async () => {
	const queryClient = new QueryClient();
	const mockTrades = [createMockTrade('1', '0', '-100')];

	const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
		subscribe: (fn: (value: unknown) => void) => {
			fn({
				data: { pages: [mockTrades] },
				status: 'success',
				isFetching: false,
				isFetched: true
			});
			return { unsubscribe: () => {} };
		}
	})) as Mock;

	render(OrderTradesListTable, {
		context: new Map([['$$_queryClient', queryClient]]),
		props: { order: mockOrder, rpcs: ['https://example.com'] }
	});

	await waitFor(() => {
		const ioRatioCell = screen.getByTestId('io-ratio');
		expect(ioRatioCell).toHaveTextContent('-');
		expect(ioRatioCell).not.toHaveTextContent('Infinity');
	});
});

test('displays dash when both amounts are zero', async () => {
	const queryClient = new QueryClient();
	const mockTrades = [createMockTrade('1', '0', '0')];

	const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
		subscribe: (fn: (value: unknown) => void) => {
			fn({
				data: { pages: [mockTrades] },
				status: 'success',
				isFetching: false,
				isFetched: true
			});
			return { unsubscribe: () => {} };
		}
	})) as Mock;

	render(OrderTradesListTable, {
		context: new Map([['$$_queryClient', queryClient]]),
		props: { order: mockOrder, rpcs: ['https://example.com'] }
	});

	await waitFor(() => {
		const ioRatioCell = screen.getByTestId('io-ratio');
		expect(ioRatioCell).toHaveTextContent('-');
		expect(ioRatioCell).not.toHaveTextContent('NaN');
	});
});

test('renders TakeOrder trade type badge and Taker label', async () => {
	const queryClient = new QueryClient();

	const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		subscribe: (fn: (value: any) => void) => {
			fn({
				data: { pages: [mockTradeOrdersList] },
				status: 'success',
				isFetching: false,
				isFetched: true
			});
			return { unsubscribe: () => {} };
		}
	})) as Mock;

	render(OrderTradesListTable, {
		context: new Map([['$$_queryClient', queryClient]]),
		props: { order: mockOrder, rpcs: ['https://example.com'] }
	});

	await waitFor(async () => {
		const rows = screen.getAllByTestId('bodyRow');
		rows.forEach((row) => {
			expect(row).toHaveTextContent('Take order');
			expect(row).toHaveTextContent('Taker:');
		});
	});
});

test('renders Clear trade with counterparty info', async () => {
	const queryClient = new QueryClient();
	const mockClearTrades = [
		{
			...mockTradeOrdersList[0],
			id: '3',
			tradeEventType: 'Clear',
			counterparty: {
				owner: '0xcounterparty_owner',
				orderHash: '0xcounterparty_order_hash'
			}
		}
	] as unknown as RaindexTrade[];

	const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		subscribe: (fn: (value: any) => void) => {
			fn({
				data: { pages: [mockClearTrades] },
				status: 'success',
				isFetching: false,
				isFetched: true
			});
			return { unsubscribe: () => {} };
		}
	})) as Mock;

	render(OrderTradesListTable, {
		context: new Map([['$$_queryClient', queryClient]]),
		props: { order: mockOrder, rpcs: ['https://example.com'] }
	});

	await waitFor(async () => {
		const rows = screen.getAllByTestId('bodyRow');
		expect(rows[0]).toHaveTextContent('Clear');
		expect(rows[0]).toHaveTextContent('Counterparty:');
		expect(rows[0]).toHaveTextContent('Order:');
	});
});

test('renders dash when counterparty is missing', async () => {
	const queryClient = new QueryClient();
	const mockNoCounterpartyTrades = [
		{
			...mockTradeOrdersList[0],
			id: '4',
			tradeEventType: 'Clear',
			counterparty: undefined
		}
	] as unknown as RaindexTrade[];

	const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		subscribe: (fn: (value: any) => void) => {
			fn({
				data: { pages: [mockNoCounterpartyTrades] },
				status: 'success',
				isFetching: false,
				isFetched: true
			});
			return { unsubscribe: () => {} };
		}
	})) as Mock;

	render(OrderTradesListTable, {
		context: new Map([['$$_queryClient', queryClient]]),
		props: { order: mockOrder, rpcs: ['https://example.com'] }
	});

	await waitFor(async () => {
		const rows = screen.getAllByTestId('bodyRow');
		expect(rows[0]).toHaveTextContent('-');
	});
});
