import type { Writable } from 'svelte/store';
import type { Address, Hex } from '@rainlanguage/raindex';

export interface AppStoresInterface {
	selectedChainIds: Writable<number[]>;
	showInactiveOrders: Writable<boolean>;
	orderHash: Writable<Hex>;
	hideZeroBalanceVaults: Writable<boolean>;
	hideInactiveOrdersVaults: Writable<boolean>;
	activeTokens: Writable<Address[]>;
	activeRaindexAddresses: Writable<Address[]>;
	ownerFilter: Writable<Address>;
}
