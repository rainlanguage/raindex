/**
 * @module awaitTransactionIndexing
 * @description Utilities for waiting for transactions to be indexed by a subgraph
 */

import { TransactionStoreErrorMessage } from '$lib/types/transaction';
import type {
	WasmEncodedResult,
	RaindexTransaction,
	RaindexOrder,
	Address,
	Hex
} from '@rainlanguage/raindex';

export type AwaitSubgraphConfig = {
	chainId: number;
	raindex: Address;
	txHash: Hex;
	successMessage: string;
	fetchEntityFn: (
		chainId: number,
		raindex: Address,
		txHash: Hex
	) => Promise<WasmEncodedResult<RaindexTransaction | RaindexOrder[] | null | undefined>>;
	isSuccess: (data: RaindexTransaction | RaindexOrder[]) => boolean;
};

/**
 * Result of a subgraph indexing operation
 * @template T The type of data returned by the subgraph
 */
export type IndexingResult<T> = {
	/**
	 * The successful result of the indexing operation
	 */
	value?: {
		/**
		 * The transaction hash
		 */
		txHash: Hex;
		/**
		 * Message to display on successful indexing
		 */
		successMessage: string;
		/**
		 * Optional order hash if available
		 */
		orderHash?: Hex;
		/**
		 * Optional network key
		 */
		network?: string;
		/**
		 * Optional data returned from the subgraph
		 */
		data?: T;
	};
	/**
	 * Error message if indexing failed
	 */
	error?: string;
};

/**
 * Generic function to handle waiting for subgraph indexing
 * Returns a promise that resolves with an object containing either value or error
 *
 * @template T The type of data returned by the subgraph
 * @param options Configuration options for the indexing operation
 * @param options.chainId Chain ID to query
 * @param options.raindex Raindex address to query
 * @param options.txHash Transaction hash to check for indexing
 * @param options.successMessage Message to display on successful indexing
 * @param options.maxAttempts Maximum number of attempts before timing out (default: 10)
 * @param options.interval Interval between attempts in milliseconds (default: 1000)
 * @param options.network Optional network identifier
 * @param options.fetchEntityFn Function to fetch data from the subgraph
 * @param options.isSuccess Function to determine if the fetched data indicates success
 * @returns Promise resolving to an IndexingResult
 */
export const awaitSubgraphIndexing = async <T>(options: {
	/**
	 * Chain ID to query
	 */
	chainId: number;
	/**
	 * Raindex address to query
	 */
	raindex: Address;
	/**
	 * Transaction hash to check for indexing
	 */
	txHash: Hex;
	/**
	 * Message to display on successful indexing
	 */
	successMessage: string;
	/**
	 * Maximum number of attempts before timing out
	 */
	maxAttempts?: number;
	/**
	 * Interval between attempts in milliseconds
	 */
	interval?: number;
	/**
	 * Optional network identifier
	 */
	network?: string;
	/**
	 * Function to fetch data from the subgraph
	 * @param chainId Chain ID to query
	 * @param raindex Raindex address to query
	 * @param txHash Transaction hash to query
	 */
	fetchEntityFn: (
		chainId: number,
		raindex: Address,
		txHash: Hex
	) => Promise<WasmEncodedResult<T | null | undefined>>;
	/**
	 * Function to determine if the fetched data indicates success
	 * @param data The data returned from the subgraph
	 */
	isSuccess: (data: T) => boolean;
}): Promise<IndexingResult<T>> => {
	const {
		chainId,
		raindex,
		txHash,
		successMessage,
		maxAttempts = 10,
		interval = 1000,
		network,
		fetchEntityFn,
		isSuccess
	} = options;

	const checkIndexing = async (attempt: number): Promise<IndexingResult<T>> => {
		try {
			const data = await fetchEntityFn(chainId, raindex, txHash);

			if (data.value && isSuccess(data.value)) {
				let newOrderHash;
				// Extract orderHash from order data if it exists in the expected format
				// This only applies to addOrder transactions
				if (Array.isArray(data.value) && data.value.length > 0 && data.value[0]?.orderHash) {
					newOrderHash = data.value[0].orderHash;
				}

				return {
					value: {
						txHash,
						successMessage,
						orderHash: newOrderHash,
						network,
						data: data.value
					}
				};
			}
		} catch {
			// Continue with the next attempt
		}

		if (attempt >= maxAttempts) {
			return {
				error: TransactionStoreErrorMessage.SUBGRAPH_TIMEOUT_ERROR
			};
		}

		await new Promise((resolve) => setTimeout(resolve, interval));
		return checkIndexing(attempt + 1);
	};

	return checkIndexing(1);
};
