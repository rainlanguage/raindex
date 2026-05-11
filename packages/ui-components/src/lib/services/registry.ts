import type { InvalidOrderDetail, ValidOrderDetail } from '$lib/types/order';
import { RaindexOrderBuilder } from '@rainlanguage/orderbook';

export type RegistryFile = {
	name: string;
	url: string;
};

export type RegistryDotrain = {
	name: string;
	dotrain: string;
};

export interface OrderValidationResult {
	validOrders: ValidOrderDetail[];
	invalidOrders: InvalidOrderDetail[];
}

/**
 * Fetches and parses a file registry from a given URL.
 * The registry is expected to be a text file where each line contains a file name and URL separated by a space.
 *
 * @param url - The URL of the registry file to fetch
 * @returns A Promise that resolves to an array of objects containing file names and their corresponding URLs
 * @throws Will throw an error if the fetch fails, if the response is not ok, or if the registry format is invalid
 *
 * @example
 * const files = await fetchParseRegistryFile('https://example.com/registry');
 * // Returns: [{ name: 'file1', url: 'https://example.com/file1.rain' }, ...]
 */

export const fetchParseRegistry = async (url: string): Promise<{ name: string; url: string }[]> => {
	try {
		const response = await fetch(url);
		if (!response.ok) {
			throw new Error('Failed to fetch registry.');
		}
		const filesList = await response.text();
		const files = filesList
			.split('\n')
			.filter((line) => line.trim())
			.map((line) => {
				const [name, url] = line.split(' ');
				return { name, url };
			});
		if (!files) {
			throw new Error('Invalid stategy registry.');
		}
		return files;
	} catch (e) {
		throw new Error(e instanceof Error ? e.message : 'Unknown error.');
	}
};

export const fetchRegistryDotrains = async (url: string): Promise<RegistryDotrain[]> => {
	const files = await fetchParseRegistry(url);
	const dotrains = await Promise.all(
		files.map(async (file) => {
			try {
				const response = await fetch(file.url);
				if (!response.ok) {
					throw new Error(`Failed to fetch dotrain for ${file.name}`);
				}
				const dotrain = await response.text();
				return { name: file.name, dotrain };
			} catch (e) {
				throw new Error(
					e instanceof Error
						? `Error fetching dotrain for ${file.name}: ${e.message}`
						: `Unknown error fetching dotrain for ${file.name}`
				);
			}
		})
	);
	return dotrains;
};

export async function validateOrders(
	registryDotrains: RegistryDotrain[]
): Promise<OrderValidationResult> {
	const ordersPromises = registryDotrains.map(async (registryDotrain) => {
		try {
			const result = await RaindexOrderBuilder.getOrderDetails(registryDotrain.dotrain);

			if (result.error) {
				throw new Error(result.error.msg);
			}

			return {
				valid: true,
				data: {
					...registryDotrain,
					details: result.value
				}
			};
		} catch (error) {
			return {
				valid: false,
				data: {
					name: registryDotrain.name,
					error: error instanceof Error ? error.message : String(error)
				}
			};
		}
	});

	const ordersResults = await Promise.all(ordersPromises);

	const validOrders = ordersResults
		.filter((result) => result.valid)
		.map((result) => result.data as ValidOrderDetail);

	const invalidOrders = ordersResults
		.filter((result) => !result.valid)
		.map((result) => result.data as InvalidOrderDetail);

	return { validOrders, invalidOrders };
}
