/* eslint-disable @typescript-eslint/no-explicit-any */
import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, type Mock, beforeEach } from 'vitest';
import VaultsListTable from '../lib/components/tables/VaultsListTable.svelte';
import { readable } from 'svelte/store';
import { Float, type RaindexVault, type RaindexVaultsList } from '@rainlanguage/orderbook';
import type { ComponentProps } from 'svelte';
import userEvent from '@testing-library/user-event';
import { useAccount } from '$lib/providers/wallet/useAccount';
import { useToasts } from '$lib/providers/toasts/useToasts';

vi.mock('$lib/providers/wallet/useAccount', () => ({
	useAccount: vi.fn()
}));

vi.mock('$lib/providers/toasts/useToasts', () => ({
	useToasts: vi.fn()
}));

vi.mock('$lib/hooks/useRaindexClient', () => ({
	useRaindexClient: vi.fn()
}));

import { useRaindexClient } from '$lib/hooks/useRaindexClient';

const mockMatchesAccount = vi.fn();
const mockAccountStore = readable('0xabcdef1234567890abcdef1234567890abcdef12');
const mockGetVaults = vi.fn();
const mockGetTokens = vi.fn();

const mockVault = {
	chainId: 1,
	id: '0x1234567890abcdef1234567890abcdef12345678',
	owner: '0xabcdef1234567890abcdef1234567890abcdef12',
	vaultId: BigInt(42),
	balance: Float.parse('1000000000000000000').value,
	formattedBalance: '1',
	token: {
		id: '0x1111111111111111111111111111111111111111',
		address: '0x1111111111111111111111111111111111111111',
		name: 'Mock Token',
		symbol: 'MTK',
		decimals: '18'
	},
	orderbook: '0x2222222222222222222222222222222222222222',
	ordersAsInput: [],
	ordersAsOutput: []
} as unknown as RaindexVault;

const mockVaultsList = {
	items: [mockVault]
} as unknown as RaindexVaultsList;

vi.mock('@tanstack/svelte-query');

// Hoisted mock stores
const {
	mockActiveNetworkRefStore,
	mockActiveOrderbookRefStore,
	mockHideZeroBalanceVaultsStore,
	mockHideInactiveOrdersVaultsStore,
	mockOrderHashStore,
	mockShowInactiveOrdersStore,
	mockActiveAccountsStore,
	mockSelectedChainIdsStore,
	mockActiveTokensStore,
	mockActiveOrderbookAddressesStore,
	mockOwnerFilterStore
} = await vi.hoisted(() => import('../lib/__mocks__/stores'));

const defaultProps = {
	orderHash: mockOrderHashStore,
	showInactiveOrders: mockShowInactiveOrdersStore,
	hideZeroBalanceVaults: mockHideZeroBalanceVaultsStore,
	hideInactiveOrdersVaults: mockHideInactiveOrdersVaultsStore,
	activeNetworkRef: mockActiveNetworkRefStore,
	activeOrderbookRef: mockActiveOrderbookRefStore,
	activeAccounts: mockActiveAccountsStore,
	selectedChainIds: mockSelectedChainIdsStore,
	activeTokens: mockActiveTokensStore,
	activeOrderbookAddresses: mockActiveOrderbookAddressesStore,
	ownerFilter: mockOwnerFilterStore
};

type VaultsListTableProps = ComponentProps<VaultsListTable>;

describe('VaultsListTable', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		(useAccount as Mock).mockReturnValue({
			matchesAccount: mockMatchesAccount,
			account: mockAccountStore
		});
		(useToasts as Mock).mockReturnValue({
			errToast: vi.fn(),
			successToast: vi.fn(),
			warningToast: vi.fn(),
			infoToast: vi.fn()
		});
		(useRaindexClient as Mock).mockReturnValue({
			getVaults: mockGetVaults,
			getTokens: mockGetTokens,
			getUniqueChainIds: vi.fn(() => ({
				value: [1],
				error: undefined
			})),
			getAllNetworks: vi.fn(() => ({
				value: new Map([[1, { name: 'Ethereum', id: 1 }]]),
				error: undefined
			})),
			getAllAccounts: vi.fn(() => ({
				value: new Map(),
				error: undefined
			})),
			getAllOrderbooks: vi.fn(() => ({
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
			}))
		});
		mockGetVaults.mockResolvedValue({ value: { items: [] }, error: undefined });
		mockGetTokens.mockResolvedValue({ value: [], error: undefined });
	});
	it('displays vault information correctly', async () => {
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [mockVaultsList] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;
		render(VaultsListTable, defaultProps as unknown as VaultsListTableProps);
		expect(screen.getByTestId('vault-network')).toHaveTextContent('Ethereum');

		const addressesCell = screen.getByTestId('vaultAddresses');
		expect(addressesCell).toBeInTheDocument();
		expect(addressesCell).toHaveTextContent('Vault:');
		expect(addressesCell).toHaveTextContent('Orderbook:');
		expect(addressesCell).toHaveTextContent('Owner:');

		// Token column now contains both token name and balance
		const tokenCell = screen.getByTestId('vault-token');
		expect(tokenCell).toHaveTextContent('Mock Token');
		expect(tokenCell).toHaveTextContent('1 MTK');
	});

	it('shows deposit/withdraw buttons when handlers are provided', async () => {
		mockMatchesAccount.mockReturnValue(true);
		const handleDepositModal = vi.fn();
		const handleWithdrawModal = vi.fn();

		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [mockVaultsList] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;
		render(VaultsListTable, {
			...defaultProps,
			handleDepositModal,
			handleWithdrawModal
		} as unknown as VaultsListTableProps);

		await waitFor(() => {
			const menuButton = screen.getByTestId('vault-menu');
			userEvent.click(menuButton);
			expect(screen.getByTestId('deposit-button')).toBeInTheDocument();
			expect(screen.getByTestId('withdraw-button')).toBeInTheDocument();
		});
	});

	it('handles deposit action', async () => {
		mockMatchesAccount.mockReturnValue(true);
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [mockVaultsList] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;
		const handleDepositModal = vi.fn();
		const handleWithdrawModal = vi.fn();

		render(VaultsListTable, {
			...defaultProps,
			handleDepositModal,
			handleWithdrawModal
		} as unknown as VaultsListTableProps);

		const menuButton = screen.getByTestId('vault-menu');
		await userEvent.click(menuButton);

		const depositButton = screen.getByTestId('deposit-button');
		await userEvent.click(depositButton);

		expect(handleDepositModal).toHaveBeenCalledWith(mockVault, undefined, new Map());
	});

	it('hides action buttons when user is not the vault owner', () => {
		mockMatchesAccount.mockReturnValue(false);
		render(VaultsListTable, {
			...defaultProps
		} as unknown as VaultsListTableProps);

		expect(screen.queryByTestId('vault-menu')).not.toBeInTheDocument();
	});

	it('displays empty state when no vaults are found', async () => {
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [{ items: [] }] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		render(VaultsListTable, defaultProps as unknown as VaultsListTableProps);
		expect(screen.getByText('No Vaults Found')).toBeInTheDocument();
	});

	it('disables selection across different networks and shows tooltip', async () => {
		mockMatchesAccount.mockReturnValue(true);

		// Create vaults on different chains
		const vault1 = { ...mockVault, chainId: 1, id: 'vault1' };
		const vault2 = { ...mockVault, chainId: 137, id: 'vault2' }; // Different chainId
		const mockVaultsListMixed = {
			items: [vault1, vault2],
			pickByIds: vi.fn((ids: string[]) => ({
				error: undefined,
				value: { items: [vault1, vault2].filter((v) => ids.includes(v.id)) }
			}))
		} as unknown as RaindexVaultsList;

		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		mockQuery.createInfiniteQuery = vi.fn(() => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [mockVaultsListMixed] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		render(VaultsListTable, defaultProps as unknown as VaultsListTableProps);

		// Wait for component to render
		await waitFor(() => {
			expect(screen.getByText('Input For')).toBeInTheDocument();
		});

		// Check that both vaults are displayed (different networks)
		const networkElements = screen.getAllByTestId('vault-network');
		expect(networkElements).toHaveLength(2);
		expect(networkElements[0]).toHaveTextContent('Ethereum'); // chainId 1

		const vaultCheckboxes = screen.getAllByTestId('vault-checkbox');
		expect(vaultCheckboxes).toHaveLength(2);

		// Select first vault to test basic selection functionality
		await userEvent.click(vaultCheckboxes[0]);
		expect(vaultCheckboxes[0]).toBeChecked();
		// Second checkbox should be disabled (different network) and show tooltip on hover
		await waitFor(() => expect(vaultCheckboxes[1]).toBeDisabled());
		await userEvent.hover(vaultCheckboxes[1]);
		await waitFor(() =>
			expect(screen.getByText('This vault is on a different network')).toBeInTheDocument()
		);
	});

	it('disables selection for zero-balance vaults and shows tooltip', async () => {
		mockMatchesAccount.mockReturnValue(true);

		// Create vault with zero balance
		const zeroBalanceVault = {
			...mockVault,
			balance: Float.parse('0').value,
			formattedBalance: '0'
		};
		const mockVaultsListZero = {
			items: [zeroBalanceVault],
			pickByIds: vi.fn((ids: string[]) => ({
				error: undefined,
				value: { items: [zeroBalanceVault].filter((v) => ids.includes(v.id)) }
			}))
		} as unknown as RaindexVaultsList;

		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		mockQuery.createInfiniteQuery = vi.fn(() => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [mockVaultsListZero] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		render(VaultsListTable, defaultProps as unknown as VaultsListTableProps);

		const vaultCheckboxes = screen.getAllByTestId('vault-checkbox');
		const firstCheckbox = vaultCheckboxes[0];
		// Wait for vault table checkbox to render and be disabled
		await waitFor(() => {
			expect(firstCheckbox).toBeDisabled();
		});

		// Hover over disabled checkbox to verify tooltip
		await userEvent.hover(firstCheckbox!);
		await waitFor(() => {
			expect(screen.getByText('This vault has a zero balance')).toBeInTheDocument();
		});
	});

	it('calls onWithdrawAll with only selected vaults', async () => {
		mockMatchesAccount.mockReturnValue(true);
		const onWithdrawAll = vi.fn();

		// Create multiple vaults on same chain
		const vault1 = { ...mockVault, id: 'vault1', chainId: 1 };
		const vault2 = { ...mockVault, id: 'vault2', chainId: 1 };
		const vault3 = { ...mockVault, id: 'vault3', chainId: 1 };

		const mockPickByIds = vi.fn((ids: string[]) => ({
			error: undefined,
			value: {
				items: [vault1, vault2, vault3].filter((v) => ids.includes(v.id)),
				concat: vi.fn(() => ({
					error: undefined,
					value: mockVaultsListMultiple
				}))
			}
		}));

		const mockVaultsListMultiple = {
			items: [vault1, vault2, vault3],
			pickByIds: mockPickByIds
		} as unknown as RaindexVaultsList;

		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		mockQuery.createInfiniteQuery = vi.fn(() => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [mockVaultsListMultiple] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		render(VaultsListTable, {
			...defaultProps,
			onWithdrawAll
		} as unknown as VaultsListTableProps);

		// Wait for vault table checkboxes to render
		await waitFor(() => {
			const vaultCheckboxes = screen.getAllByTestId('vault-checkbox');
			expect(vaultCheckboxes).toHaveLength(3);
		});

		const vaultCheckboxes = screen.getAllByTestId('vault-checkbox');

		// Select first two vaults
		await userEvent.click(vaultCheckboxes[0]);
		await userEvent.click(vaultCheckboxes[1]);

		// Click "Withdraw selected" button
		const withdrawButton = screen.getByTestId('withdraw-all-button');
		expect(withdrawButton).toHaveTextContent('Withdraw selected (2)');
		await userEvent.click(withdrawButton);

		// Verify onWithdrawAll was called
		expect(onWithdrawAll).toHaveBeenCalledTimes(1);

		// Verify it was called with raindexClient and filtered vaultsList
		const [clientArg, vaultsListArg] = onWithdrawAll.mock.calls[0];
		expect(clientArg).toBeDefined();
		expect(vaultsListArg).toBeDefined();

		// Verify pickByIds was called with correct vault IDs
		expect(mockPickByIds).toHaveBeenCalledWith(['vault1', 'vault2']);
	});
	it('shows correct button text when no vaults are selected', async () => {
		const onWithdrawAll = vi.fn();

		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		mockQuery.createInfiniteQuery = vi.fn(() => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [mockVaultsList] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		render(VaultsListTable, {
			...defaultProps,
			onWithdrawAll
		} as unknown as VaultsListTableProps);

		const withdrawButton = screen.getByTestId('withdraw-all-button');
		expect(withdrawButton).toHaveTextContent('Withdraw vaults');
		expect(withdrawButton).toBeDisabled();
	});

	it('passes orderbookAddresses filter to getVaults when orderbooks are selected', async () => {
		const orderbookAddress = '0x1111111111111111111111111111111111111111';

		mockActiveOrderbookAddressesStore.mockSetSubscribeValue([orderbookAddress]);

		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		mockQuery.createInfiniteQuery = vi.fn((options: any) => {
			if (options.queryFn) {
				options.queryFn({ pageParam: 0 });
			}
			return {
				subscribe: (fn: (value: any) => void) => {
					fn({
						data: { pages: [{ items: [] }] },
						status: 'success',
						isFetching: false,
						isFetched: true
					});
					return { unsubscribe: () => {} };
				}
			};
		}) as Mock;

		render(VaultsListTable, defaultProps as unknown as VaultsListTableProps);

		await waitFor(() => {
			expect(mockGetVaults).toHaveBeenCalledWith(
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
		mockQuery.createInfiniteQuery = vi.fn((options: any) => {
			if (options.queryFn) {
				options.queryFn({ pageParam: 0 });
			}
			return {
				subscribe: (fn: (value: any) => void) => {
					fn({
						data: { pages: [{ items: [] }] },
						status: 'success',
						isFetching: false,
						isFetched: true
					});
					return { unsubscribe: () => {} };
				}
			};
		}) as Mock;

		render(VaultsListTable, defaultProps as unknown as VaultsListTableProps);

		await waitFor(() => {
			expect(mockGetVaults).toHaveBeenCalledWith(
				expect.anything(),
				expect.objectContaining({
					orderbookAddresses: undefined
				}),
				expect.anything()
			);
		});
	});
});
