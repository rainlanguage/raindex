import { writable } from 'svelte/store';
import type { Address } from '@rainlanguage/orderbook';
import { type Config } from '@wagmi/core';
import { mockWeb3Config } from './mockWeb3Config';

type MockDeployment = { key: string; name?: string; description?: string } | null;

type MockPageState = {
	data: Record<string, unknown> & { deployment: MockDeployment };
	url: URL;
	params: Record<string, unknown>;
	form: Record<string, unknown>;
	status: number;
	error: unknown;
	route: {
		id: string | null;
	};
};

if (import.meta.vitest) {
	vi.mock(import('@rainlanguage/orderbook'), async (importOriginal) => {
		const actual = await importOriginal();
		return {
			...actual
		};
	});
}

const initialPageState: MockPageState = {
	data: {
		dotrain: 'some dotrain content',
		deployment: { key: 'deploy-key', name: '', description: '' },
		orderDetail: {}
	},
	url: new URL('http://localhost:3000/deploy'),
	params: {},
	form: {},
	status: 200,
	error: null,
	route: {
		id: null
	}
};

const mockPageWritable = writable<MockPageState>(initialPageState);
const mockShowInactiveOrdersWritable = writable<boolean>(true);
const mockOrderHashWritable = writable<string>('');
const mockHideZeroBalanceVaultsWritable = writable<boolean>(false);
const mockHideInactiveOrdersVaultsWritable = writable<boolean>(false);
const mockActiveNetworkRefWritable = writable<string>('');
const mockActiveOrderbookRefWritable = writable<string>('');
const mockActiveAccountsWritable = writable<Record<string, string>>({});
const mockSubgraphUrlWritable = writable<string>('');
const mockChainIdWritable = writable<number>(0);
const mockConnectedWritable = writable<boolean>(true);
const mockWagmiConfigWritable = writable<Config>(mockWeb3Config);
const mockSelectedChainIdsWritable = writable<number[]>([]);
const mockActiveTokensWritable = writable<string[]>([]);
const mockActiveOrderbookAddressesWritable = writable<string[]>([]);
const mockOwnerFilterWritable = writable<Address>('' as Address);

export const mockShowInactiveOrdersStore = {
	subscribe: mockShowInactiveOrdersWritable.subscribe,
	set: mockShowInactiveOrdersWritable.set,
	mockSetSubscribeValue: (value: boolean): void => mockShowInactiveOrdersWritable.set(value)
};

export const mockOrderHashStore = {
	subscribe: mockOrderHashWritable.subscribe,
	set: mockOrderHashWritable.set,
	mockSetSubscribeValue: (value: string): void => mockOrderHashWritable.set(value)
};

export const mockHideZeroBalanceVaultsStore = {
	subscribe: mockHideZeroBalanceVaultsWritable.subscribe,
	set: mockHideZeroBalanceVaultsWritable.set,
	mockSetSubscribeValue: (value: boolean): void => mockHideZeroBalanceVaultsWritable.set(value)
};

export const mockHideInactiveOrdersVaultsStore = {
	subscribe: mockHideInactiveOrdersVaultsWritable.subscribe,
	set: mockHideInactiveOrdersVaultsWritable.set,
	mockSetSubscribeValue: (value: boolean): void => mockHideInactiveOrdersVaultsWritable.set(value)
};

export const mockActiveNetworkRefStore = {
	subscribe: mockActiveNetworkRefWritable.subscribe,
	set: mockActiveNetworkRefWritable.set,
	mockSetSubscribeValue: (value: string): void => mockActiveNetworkRefWritable.set(value)
};

export const mockActiveOrderbookRefStore = {
	subscribe: mockActiveOrderbookRefWritable.subscribe,
	set: mockActiveOrderbookRefWritable.set,
	mockSetSubscribeValue: (value: string): void => mockActiveOrderbookRefWritable.set(value)
};

export const mockActiveAccountsStore = {
	subscribe: mockActiveAccountsWritable.subscribe,
	set: mockActiveAccountsWritable.set,
	mockSetSubscribeValue: (value: Record<string, string>): void =>
		mockActiveAccountsWritable.set(value)
};

export const mockSubgraphUrlStore = {
	subscribe: mockSubgraphUrlWritable.subscribe,
	set: mockSubgraphUrlWritable.set,
	mockSetSubscribeValue: (value: string): void => mockSubgraphUrlWritable.set(value)
};

export const mockChainIdStore = {
	subscribe: mockChainIdWritable.subscribe,
	set: mockChainIdWritable.set,
	mockSetSubscribeValue: (value: number): void => mockChainIdWritable.set(value)
};

export const mockConnectedStore = {
	subscribe: mockConnectedWritable.subscribe,
	set: mockConnectedWritable.set,
	update: mockConnectedWritable.update,
	mockSetSubscribeValue: (value: boolean): void => mockConnectedWritable.set(value)
};

export const mockWagmiConfigStore = {
	subscribe: mockWagmiConfigWritable.subscribe,
	set: mockWagmiConfigWritable.set,
	update: mockWagmiConfigWritable.update,
	mockSetSubscribeValue: (value: Config): void => mockWagmiConfigWritable.set(value)
};

export const mockSelectedChainIdsStore = {
	subscribe: mockSelectedChainIdsWritable.subscribe,
	set: mockSelectedChainIdsWritable.set,
	mockSetSubscribeValue: (value: number[]): void => mockSelectedChainIdsWritable.set(value)
};

export const mockPageStore = {
	subscribe: mockPageWritable.subscribe,
	set: mockPageWritable.set,
	mockSetSubscribeValue: (newValue: Partial<MockPageState>): void => {
		mockPageWritable.update((currentValue) => ({
			...currentValue,
			...newValue
		}));
	},
	reset: () => mockPageWritable.set(initialPageState)
};

export const mockActiveTokensStore = {
	subscribe: mockActiveTokensWritable.subscribe,
	set: mockActiveTokensWritable.set,
	mockSetSubscribeValue: (value: string[]): void => mockActiveTokensWritable.set(value)
};

export const mockActiveOrderbookAddressesStore = {
	subscribe: mockActiveOrderbookAddressesWritable.subscribe,
	set: mockActiveOrderbookAddressesWritable.set,
	mockSetSubscribeValue: (value: string[]): void => mockActiveOrderbookAddressesWritable.set(value)
};

export const mockOwnerFilterStore = {
	subscribe: mockOwnerFilterWritable.subscribe,
	set: mockOwnerFilterWritable.set,
	mockSetSubscribeValue: (value: Address): void => mockOwnerFilterWritable.set(value)
};
