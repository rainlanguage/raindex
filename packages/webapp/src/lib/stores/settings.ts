import { cachedWritableStore } from '@rainlanguage/ui-components';
import { type Address, type Hex } from '@rainlanguage/orderbook';
import { get, writable } from 'svelte/store';

/**
 * A persistent store that controls whether vaults with zero balance should be hidden in the UI.
 *
 * This setting is saved to local storage and persists between sessions.
 *
 * @default true - Zero balance vaults are hidden by default
 * @returns A writable store containing a boolean value
 */
export const hideZeroBalanceVaults = cachedWritableStore<boolean>(
	'settings.hideZeroBalanceVaults',
	true, // default value is true
	(value) => JSON.stringify(value),
	(str) => {
		try {
			const value = JSON.parse(str);
			return typeof value === 'boolean' ? value : true;
		} catch {
			return true;
		}
	}
);

/**
 * A persistent store that holds an owner address to filter orders/vaults by.
 *
 * This setting is saved to local storage and persists between sessions.
 *
 * @default '' - Empty string means no owner filtering
 * @returns A writable store containing an address string
 */
export const ownerFilter = cachedWritableStore<Address>(
	'settings.ownerFilter',
	// @ts-expect-error initially the value is empty
	'',
	(value) => value,
	(str) => (str || '') as Address
);

/**
 * A persistent store that holds the currently selected chain IDs.
 *
 * This setting is saved to local storage and persists between sessions.
 *
 * @default [] - Empty array by default
 * @returns A writable store containing an array of chain IDs
 */
export const selectedChainIds = cachedWritableStore<number[]>(
	'settings.selectedChainIds',
	[],
	(value) => JSON.stringify(value),
	(str) => JSON.parse(str)
);

/**
 * Store holding the list of valid/available chain IDs from the RaindexClient.
 * Used to automatically filter selectedChainIds to ensure only valid IDs are selected.
 * Populated during app initialization from RaindexClient.getUniqueChainIds().
 * @default [] - Empty array until populated
 */
export const validChainIds = writable<number[]>([]);

validChainIds.subscribe((valid) => {
	if (valid.length > 0) {
		const current = get(selectedChainIds);
		const filtered = current.filter((id) => valid.includes(id));
		if (filtered.length !== current.length) {
			selectedChainIds.set(filtered);
		}
	}
});

/**
 * A persistent store that holds the currently selected order hash.
 *
 * This setting is saved to local storage and persists between sessions.
 *
 * @default "" - Empty string by default
 * @returns A writable store containing the order hash string
 */
export const orderHash = cachedWritableStore<Hex>(
	'settings.orderHash',
	// @ts-expect-error initially the value is empty
	'',
	(value) => value,
	(str) => (str || '') as Hex
);

/**
 * A persistent store that holds the currently show/hide setting for inactive orders.
 *
 * This setting is saved to local storage and persists between sessions.
 *
 * @default false - Inactive orders are hidden by default
 * @returns A writable store containing a boolean value
 */
export const showInactiveOrders = cachedWritableStore<boolean>(
	'settings.showInactiveOrders',
	false,
	(value) => JSON.stringify(value),
	(str) => {
		try {
			const value = JSON.parse(str);
			return typeof value === 'boolean' ? value : false;
		} catch {
			return false;
		}
	}
);

/**
 * A persistent store that holds the currently selected tokens for filtering.
 *
 * This setting is saved to local storage and persists between sessions.
 *
 * @default [] - Empty array by default
 * @returns A writable store containing selected tokens mapped by address
 */
export const activeTokens = cachedWritableStore<Address[]>(
	'settings.selectedTokens',
	[],
	JSON.stringify,
	(str) => {
		try {
			return JSON.parse(str);
		} catch {
			return [];
		}
	}
);

/**
 * A persistent store that holds the currently selected orderbook addresses for filtering.
 *
 * This setting is saved to local storage and persists between sessions.
 *
 * @default [] - Empty array by default
 * @returns A writable store containing selected orderbook addresses mapped by address
 */
export const activeOrderbookAddresses = cachedWritableStore<Address[]>(
	'settings.activeOrderbookAddresses',
	[],
	JSON.stringify,
	(str) => {
		try {
			return JSON.parse(str);
		} catch {
			return [];
		}
	}
);

/**
 * A persistent store that controls whether vaults without active orders should be hidden in the UI.
 *
 * This setting is saved to local storage and persists between sessions.
 *
 * @default false - Vaults without active orders are shown by default
 * @returns A writable store containing a boolean value
 */
export const hideInactiveOrdersVaults = cachedWritableStore<boolean>(
	'settings.hideInactiveOrdersVaults',
	false,
	(value) => JSON.stringify(value),
	(str) => {
		try {
			const value = JSON.parse(str);
			return typeof value === 'boolean' ? value : false;
		} catch {
			return false;
		}
	}
);
