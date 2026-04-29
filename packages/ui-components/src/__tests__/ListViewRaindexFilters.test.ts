import { render, screen } from '@testing-library/svelte';
import { readable, writable } from 'svelte/store';
import { beforeEach, expect, test, describe, vi, type Mock } from 'vitest';
import ListViewRaindexFilters from '../lib/components/ListViewRaindexFilters.svelte';
import type { Address, RaindexVaultToken } from '@rainlanguage/raindex';
import type { ComponentProps } from 'svelte';
import type { QueryObserverResult } from '@tanstack/svelte-query';
import { useRaindexClient } from '$lib/hooks/useRaindexClient';

const { mockPageStore } = await vi.hoisted(() => import('$lib/__mocks__/stores.ts'));

const mockAccount = writable(null);

vi.mock('$lib/providers/wallet/useAccount', () => ({
	useAccount: () => ({
		account: mockAccount,
		matchesAccount: vi.fn()
	})
}));

vi.mock('$app/stores', () => ({
	page: mockPageStore
}));

vi.mock('@tanstack/svelte-query', () => ({
	createInfiniteQuery: vi.fn()
}));

vi.mock('$lib/hooks/useRaindexClient', () => ({
	useRaindexClient: vi.fn()
}));

type ListViewRaindexFiltersProps = ComponentProps<ListViewRaindexFilters>;

describe('ListViewRaindexFilters', () => {
	const mockGetAllRaindexes = vi.fn();

	const defaultProps: ListViewRaindexFiltersProps = {
		hideZeroBalanceVaults: writable(false),
		hideInactiveOrdersVaults: writable(false),
		selectedChainIds: writable([]),
		showInactiveOrders: writable(true),
		orderHash: writable('0x0234'),
		activeTokens: writable([]),
		selectedTokens: [],
		tokensQuery: readable({
			isLoading: false,
			isError: false,
			data: [] as RaindexVaultToken[],
			error: null
		} as QueryObserverResult<RaindexVaultToken[], Error>),
		activeRaindexAddresses: writable<Address[]>([]),
		selectedRaindexAddresses: [],
		ownerFilter: writable<Address>('' as unknown as Address)
	} as ListViewRaindexFiltersProps;

	beforeEach(() => {
		mockGetAllRaindexes.mockReturnValue({
			value: new Map(),
			error: undefined
		});

		(useRaindexClient as Mock).mockReturnValue({
			getUniqueChainIds: vi.fn(() => ({
				value: [1],
				error: undefined
			})),
			getAllNetworks: vi.fn(() => ({
				value: new Map([
					[
						'ethereum',
						{
							key: 'ethereum',
							rpcs: ['https://rpc.ankr.com/eth'],
							chainId: 1,
							networkId: 1,
							currency: 'ETH'
						}
					]
				]),
				error: undefined
			})),
			getAllRaindexes: mockGetAllRaindexes
		});

		mockAccount.set(null);
	});

	test('shows no networks alert when networks are empty', () => {
		(useRaindexClient as Mock).mockReturnValue({
			getAllNetworks: vi.fn(() => ({
				value: new Map(),
				error: undefined
			}))
		});
		render(ListViewRaindexFilters, defaultProps);

		expect(screen.getByTestId('no-networks-alert')).toBeInTheDocument();
	});

	test('shows vault-specific components on vault page', () => {
		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/vaults'
			} as URL
		});
		render(ListViewRaindexFilters, defaultProps);

		expect(screen.getByTestId('zero-balance-vault-checkbox')).toBeInTheDocument();
		expect(screen.getByTestId('inactive-orders-vault-checkbox')).toBeInTheDocument();
		expect(screen.queryByTestId('order-hash-input')).not.toBeInTheDocument();
		expect(screen.queryByTestId('order-status-checkbox')).not.toBeInTheDocument();
	});

	test('shows order-specific components on orders page', () => {
		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/orders'
			} as URL
		});
		render(ListViewRaindexFilters, defaultProps);

		expect(screen.getByTestId('order-hash-input')).toBeInTheDocument();
		expect(screen.getByTestId('order-status-checkbox')).toBeInTheDocument();
		expect(screen.queryByTestId('zero-balance-vault-checkbox')).not.toBeInTheDocument();
		expect(screen.queryByTestId('inactive-orders-vault-checkbox')).not.toBeInTheDocument();
	});

	test('shows common components when networks exist', () => {
		render(ListViewRaindexFilters, defaultProps);

		expect(screen.getByTestId('subgraphs-dropdown')).toBeInTheDocument();
	});

	test('does not show page-specific components on default view', () => {
		mockPageStore.reset();
		render(ListViewRaindexFilters, defaultProps);

		expect(screen.queryByTestId('zero-balance-vault-checkbox')).not.toBeInTheDocument();
		expect(screen.queryByTestId('inactive-orders-vault-checkbox')).not.toBeInTheDocument();
		expect(screen.queryByTestId('order-hash-input')).not.toBeInTheDocument();
		expect(screen.queryByTestId('order-status-checkbox')).not.toBeInTheDocument();
	});

	test('shows owner filter input', () => {
		render(ListViewRaindexFilters, defaultProps);

		expect(screen.getByTestId('owner-filter-input')).toBeInTheDocument();
	});

	test('shows raindexes dropdown when raindexes exist', () => {
		mockGetAllRaindexes.mockReturnValue({
			value: new Map([
				[
					'raindex1',
					{
						key: 'raindex1',
						address: '0x1234567890123456789012345678901234567890',
						label: 'Test Raindex',
						network: { chainId: 1 }
					}
				]
			]),
			error: undefined
		});

		(useRaindexClient as Mock).mockReturnValue({
			getUniqueChainIds: vi.fn(() => ({
				value: [1],
				error: undefined
			})),
			getAllNetworks: vi.fn(() => ({
				value: new Map([
					[
						'ethereum',
						{
							key: 'ethereum',
							rpcs: ['https://rpc.ankr.com/eth'],
							chainId: 1,
							networkId: 1,
							currency: 'ETH'
						}
					]
				]),
				error: undefined
			})),
			getAllRaindexes: mockGetAllRaindexes
		});

		render(ListViewRaindexFilters, defaultProps);

		expect(screen.getByTestId('dropdown-raindexes-filter-button')).toBeInTheDocument();
	});

	test('shows raindexes dropdown even when no raindexes exist', () => {
		mockGetAllRaindexes.mockReturnValue({
			value: new Map(),
			error: undefined
		});

		render(ListViewRaindexFilters, defaultProps);

		expect(screen.getByTestId('dropdown-raindexes-filter-button')).toBeInTheDocument();
	});
});
