/* eslint-disable @typescript-eslint/no-explicit-any */
import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, type Mock } from 'vitest';
import userEvent from '@testing-library/user-event';
import OrdersListTable from '../lib/components/tables/OrdersListTable.svelte';
import { RaindexOrder } from '@rainlanguage/orderbook';
import type { ComponentProps } from 'svelte';
import { readable } from 'svelte/store';
import { useAccount } from '$lib/providers/wallet/useAccount';

vi.mock('../lib/components/ListViewOrderbookFilters.svelte', async () => {
	const MockComponent = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return {
		default: MockComponent
	};
});

vi.mock('$lib/providers/wallet/useAccount', () => ({
	useAccount: vi.fn()
}));

vi.mock('$lib/hooks/useRaindexClient', () => ({
	useRaindexClient: vi.fn()
}));

import { useRaindexClient } from '$lib/hooks/useRaindexClient';

const mockAccountStore = readable('0xabcdef1234567890abcdef1234567890abcdef12');
const mockMatchesAccount = vi.fn();

const mockVaultsList = () => ({
	items: [],
	getWithdrawableVaults: () => ({ value: [], error: null })
});

const mockOrder = {
	chainId: 1,
	id: '0x1234567890abcdef1234567890abcdef12345678',
	orderBytes: '',
	orderHash: '0x4444444444444444444444444444444444444444',
	owner: '0xabcdef1234567890abcdef1234567890abcdef12',
	inputs: [
		{
			token: {
				symbol: 'ETH'
			},
			formattedBalance: '1.5'
		}
	],
	outputs: [
		{
			token: {
				symbol: 'DAI'
			},
			formattedBalance: '2500.0'
		}
	],
	vaults: [],
	inputsList: {
		...mockVaultsList(),
		items: [
			{
				token: {
					symbol: 'ETH'
				},
				formattedBalance: '1.5'
			}
		]
	},
	outputsList: {
		...mockVaultsList(),
		items: [
			{
				token: {
					symbol: 'DAI'
				},
				formattedBalance: '2500.0'
			}
		]
	},
	inputsOutputsList: mockVaultsList(),
	vaultsList: mockVaultsList(),
	orderbook: '0x2222222222222222222222222222222222222222',
	active: true,
	timestampAdded: BigInt(1678901234),
	meta: '',
	rainlang: '',
	tradesCount: 2
} as unknown as RaindexOrder;

vi.mock('@tanstack/svelte-query');

// Hoisted mock stores
const {
	mockHideZeroBalanceVaultsStore,
	mockHideInactiveOrdersVaultsStore,
	mockOrderHashStore,
	mockShowInactiveOrdersStore,
	mockSelectedChainIdsStore,
	mockActiveTokensStore,
	mockActiveOrderbookAddressesStore,
	mockOwnerFilterStore
} = await vi.hoisted(() => import('../lib/__mocks__/stores'));

type OrdersListTableProps = ComponentProps<OrdersListTable>;

const defaultProps: OrdersListTableProps = {
	showInactiveOrders: mockShowInactiveOrdersStore,
	orderHash: mockOrderHashStore,
	hideZeroBalanceVaults: mockHideZeroBalanceVaultsStore,
	hideInactiveOrdersVaults: mockHideInactiveOrdersVaultsStore,
	selectedChainIds: mockSelectedChainIdsStore,
	activeTokens: mockActiveTokensStore,
	activeOrderbookAddresses: mockActiveOrderbookAddressesStore,
	ownerFilter: mockOwnerFilterStore
} as unknown as OrdersListTableProps;

const mockGetOrders = vi.fn();
const mockGetTokens = vi.fn();
const mockGetAllOrderbooks = vi.fn();

describe('OrdersListTable', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		(useAccount as Mock).mockReturnValue({
			account: mockAccountStore,
			matchesAccount: mockMatchesAccount
		});
		(useRaindexClient as Mock).mockReturnValue({
			getOrders: mockGetOrders,
			getTokens: mockGetTokens,
			getAllOrderbooks: mockGetAllOrderbooks.mockReturnValue({
				value: new Map([
					[
						'orderbook1',
						{
							key: 'orderbook1',
							address: '0x1111111111111111111111111111111111111111',
							network: { chainId: 1 }
						}
					]
				]),
				error: undefined
			})
		});
		mockGetOrders.mockResolvedValue({ value: { orders: [], totalCount: 0 }, error: undefined });
		mockGetTokens.mockResolvedValue({ value: [], error: undefined });
	});

	it('displays order information correctly', async () => {
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [{ orders: [mockOrder], totalCount: 1 }] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;
		render(OrdersListTable, defaultProps as OrdersListTableProps);

		const orderInfoCell = screen.getByTestId('orderListRowOrderInfo');
		expect(orderInfoCell).toHaveTextContent('Ethereum');
		expect(orderInfoCell).toHaveTextContent('Active');
		expect(orderInfoCell).toHaveTextContent('Added:');

		const addressesCell = screen.getByTestId('orderListRowAddresses');
		expect(addressesCell).toBeInTheDocument();
		expect(addressesCell).toHaveTextContent('Order:');
		expect(addressesCell).toHaveTextContent('Owner:');
		expect(addressesCell).toHaveTextContent('Orderbook:');

		// Check that vault cards are rendered with correct content
		const vaultCards = screen.getAllByTestId('vault-card');
		expect(vaultCards).toHaveLength(2); // One input, one output
		expect(screen.getByText('ETH')).toBeInTheDocument();
		expect(screen.getByText('1.5')).toBeInTheDocument();
		expect(screen.getByText('DAI')).toBeInTheDocument();
		expect(screen.getByText('2500.0')).toBeInTheDocument();

		expect(screen.getByTestId('orderListRowTrades')).toHaveTextContent('2');
	});

	it('displays token information in compact layout', async () => {
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [{ orders: [mockOrder], totalCount: 1 }] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;
		render(OrdersListTable, defaultProps as OrdersListTableProps);

		// Verify token symbols and balances are displayed in vault cards
		expect(screen.getByText('ETH')).toBeInTheDocument();
		expect(screen.getByText('1.5')).toBeInTheDocument();
		expect(screen.getByText('DAI')).toBeInTheDocument();
		expect(screen.getByText('2500.0')).toBeInTheDocument();

		// Verify "Strategy Balance:" label is not present (since we're using vault cards now)
		expect(screen.queryByText('Strategy Balance:')).not.toBeInTheDocument();
	});

	it('displays multiple tokens correctly in grid layout with shared IO', async () => {
		const orderWithMultipleTokens = {
			...mockOrder,
			inputs: [
				{
					token: { symbol: 'ETH' },
					formattedBalance: '1.5'
				},
				{
					token: { symbol: 'USDC' },
					formattedBalance: '100.0'
				}
			],
			outputs: [
				{
					token: { symbol: 'DAI' },
					formattedBalance: '2500.0'
				},
				{
					token: { symbol: 'USDC' },
					formattedBalance: '100.0'
				}
			],
			inputsList: {
				...mockVaultsList(),
				items: [
					{
						id: '0xeth',
						token: { symbol: 'ETH' },
						formattedBalance: '1.5'
					}
				]
			},
			outputsList: {
				...mockVaultsList(),
				items: [
					{
						id: '0xdai',
						token: { symbol: 'DAI' },
						formattedBalance: '2500.0'
					}
				]
			},
			inputsOutputsList: {
				...mockVaultsList(),
				items: [
					{
						id: '0xusdc',
						token: { symbol: 'USDC' },
						formattedBalance: '100.0'
					}
				]
			}
		};

		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [{ orders: [orderWithMultipleTokens], totalCount: 1 }] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;
		render(OrdersListTable, defaultProps as OrdersListTableProps);

		// Verify all tokens are displayed in vault cards
		const vaultCards = screen.getAllByTestId('vault-card');
		expect(vaultCards).toHaveLength(4); // 2 inputs + 2 outputs (1 is shared between IO)

		// Verify all input tokens are displayed
		expect(screen.getByText('ETH')).toBeInTheDocument();
		expect(screen.getByText('1.5')).toBeInTheDocument();

		// Verify all output tokens are displayed
		expect(screen.getByText('DAI')).toBeInTheDocument();
		expect(screen.getByText('2500.0')).toBeInTheDocument();

		// Verify shared token is displayed for input and output
		expect(screen.getAllByText('USDC')).toHaveLength(2);
		expect(screen.getAllByText('100.0')).toHaveLength(2);
	});

	it('displays multiple tokens correctly in grid layout', async () => {
		const orderWithMultipleTokens = {
			...mockOrder,
			inputs: [
				{
					token: { symbol: 'ETH' },
					formattedBalance: '1.5'
				},
				{
					token: { symbol: 'USDC' },
					formattedBalance: '100.0'
				}
			],
			outputs: [
				{
					token: { symbol: 'DAI' },
					formattedBalance: '2500.0'
				},
				{
					token: { symbol: 'WBTC' },
					formattedBalance: '0.05'
				}
			],
			inputsList: {
				...mockVaultsList(),
				items: [
					{
						token: { symbol: 'ETH' },
						formattedBalance: '1.5'
					},
					{
						token: { symbol: 'USDC' },
						formattedBalance: '100.0'
					}
				]
			},
			outputsList: {
				...mockVaultsList(),
				items: [
					{
						token: { symbol: 'DAI' },
						formattedBalance: '2500.0'
					},
					{
						token: { symbol: 'WBTC' },
						formattedBalance: '0.05'
					}
				]
			}
		};

		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [{ orders: [orderWithMultipleTokens], totalCount: 1 }] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;
		render(OrdersListTable, defaultProps as OrdersListTableProps);

		// Verify all tokens are displayed in vault cards
		const vaultCards = screen.getAllByTestId('vault-card');
		expect(vaultCards).toHaveLength(4); // 2 inputs + 2 outputs

		// Verify all input tokens are displayed
		expect(screen.getByText('ETH')).toBeInTheDocument();
		expect(screen.getByText('1.5')).toBeInTheDocument();
		expect(screen.getByText('USDC')).toBeInTheDocument();
		expect(screen.getByText('100.0')).toBeInTheDocument();

		// Verify all output tokens are displayed
		expect(screen.getByText('DAI')).toBeInTheDocument();
		expect(screen.getByText('2500.0')).toBeInTheDocument();
		expect(screen.getByText('WBTC')).toBeInTheDocument();
		expect(screen.getByText('0.05')).toBeInTheDocument();
	});

	it('shows inactive badge for inactive orders', async () => {
		const inactiveOrder = {
			...mockOrder,
			active: false
		};
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [{ orders: [inactiveOrder], totalCount: 1 }] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;
		render(OrdersListTable, defaultProps as OrdersListTableProps);

		const orderInfoCell = screen.getByTestId('orderListRowOrderInfo');
		expect(orderInfoCell).toHaveTextContent('Inactive');
	});

	it('displays empty state when no orders are found', async () => {
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [{ orders: [], totalCount: 0 }] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		render(OrdersListTable, defaultProps as OrdersListTableProps);
		expect(screen.getByText('No Orders Found')).toBeInTheDocument();
	});

	it('navigates to order details on row click', async () => {
		vi.mock('$app/navigation', () => ({
			goto: vi.fn()
		}));

		const gotoMock = await import('$app/navigation');
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [{ orders: [mockOrder], totalCount: 1 }] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		render(OrdersListTable, defaultProps as OrdersListTableProps);

		// Simulate row click
		const event = new CustomEvent('clickRow', {
			detail: {
				item: mockOrder
			}
		});

		// Find the AppTable component and dispatch the event
		const appTable = document.querySelector('div[role="table"]');
		if (appTable) {
			appTable.dispatchEvent(event);
			expect(gotoMock.goto).toHaveBeenCalledWith(
				`/orders/${mockOrder.chainId}-${mockOrder.orderbook}-${mockOrder.orderHash}`
			);
		}
	});

	it('handles large number of trades display', async () => {
		const orderWithManyTrades = {
			...mockOrder,
			tradesCount: 100
		};

		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [{ orders: [orderWithManyTrades], totalCount: 1 }] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		render(OrdersListTable, defaultProps as OrdersListTableProps);
		expect(screen.getByTestId('orderListRowTrades')).toHaveTextContent('>99');
	});

	it('shows Take button for active orders', async () => {
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [{ orders: [mockOrder], totalCount: 1 }] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		render(OrdersListTable, {
			...defaultProps,
			handleTakeOrderModal: vi.fn()
		} as OrdersListTableProps);

		expect(screen.getByTestId(`order-take-${mockOrder.id}`)).toBeInTheDocument();
		expect(screen.getByTestId(`order-take-${mockOrder.id}`)).toHaveTextContent('Take');
	});

	it('does not show Take button for inactive orders', async () => {
		const inactiveOrder = {
			...mockOrder,
			active: false
		};
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [{ orders: [inactiveOrder], totalCount: 1 }] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		render(OrdersListTable, {
			...defaultProps,
			handleTakeOrderModal: vi.fn()
		} as OrdersListTableProps);

		expect(screen.queryByTestId(`order-take-${inactiveOrder.id}`)).not.toBeInTheDocument();
	});

	it('calls handleTakeOrderModal when Take button clicked', async () => {
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		const mockRefetch = vi.fn();
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [{ orders: [mockOrder], totalCount: 1 }] },
					status: 'success',
					isFetching: false,
					isFetched: true,
					refetch: mockRefetch
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		const handleTakeOrderModal = vi.fn();

		render(OrdersListTable, {
			...defaultProps,
			handleTakeOrderModal
		} as OrdersListTableProps);

		const takeButton = screen.getByTestId(`order-take-${mockOrder.id}`);
		await userEvent.click(takeButton);

		expect(handleTakeOrderModal).toHaveBeenCalledWith(mockOrder, mockRefetch, new Map());
	});

	it('passes orderbookAddresses filter to getOrders when orderbooks are selected', async () => {
		const orderbookAddress = '0x1111111111111111111111111111111111111111';

		mockActiveOrderbookAddressesStore.mockSetSubscribeValue([orderbookAddress]);

		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		mockQuery.createInfiniteQuery = vi.fn((options: any) => {
			if (options.queryFn) {
				options.queryFn({ pageParam: 0 });
			}
			return {
				subscribe: (fn: (value: any) => void) => {
					fn({
						data: { pages: [{ orders: [], totalCount: 0 }] },
						status: 'success',
						isFetching: false,
						isFetched: true
					});
					return { unsubscribe: () => {} };
				}
			};
		}) as Mock;

		render(OrdersListTable, defaultProps as OrdersListTableProps);

		await waitFor(() => {
			expect(mockGetOrders).toHaveBeenCalledWith(
				expect.anything(),
				expect.objectContaining({
					orderbookAddresses: [orderbookAddress]
				}),
				expect.anything()
			);
		});
	});

	it('does not pass orderbookAddresses filter when no orderbooks are selected', async () => {
		mockActiveOrderbookAddressesStore.mockSetSubscribeValue([]);

		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		mockQuery.createInfiniteQuery = vi.fn((options: any) => {
			if (options.queryFn) {
				options.queryFn({ pageParam: 0 });
			}
			return {
				subscribe: (fn: (value: any) => void) => {
					fn({
						data: { pages: [{ orders: [], totalCount: 0 }] },
						status: 'success',
						isFetching: false,
						isFetched: true
					});
					return { unsubscribe: () => {} };
				}
			};
		}) as Mock;

		render(OrdersListTable, defaultProps as OrdersListTableProps);

		await waitFor(() => {
			expect(mockGetOrders).toHaveBeenCalledWith(
				expect.anything(),
				expect.objectContaining({
					orderbookAddresses: undefined
				}),
				expect.anything()
			);
		});
	});
});
