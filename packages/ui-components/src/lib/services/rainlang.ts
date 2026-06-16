import type { InvalidOrderDetail, ValidOrderDetail } from '$lib/types/order';
import { DotrainOrderGui } from '@rainlanguage/orderbook';

export type RainlangFile = {
	name: string;
	url: string;
};

export type RainlangDotrain = {
	name: string;
	dotrain: string;
};

export interface OrderValidationResult {
	validOrders: ValidOrderDetail[];
	invalidOrders: InvalidOrderDetail[];
}

/**
 * Fetches and parses a file rainlang from a given URL.
 * The rainlang is expected to be a text file where each line contains a file name and URL separated by a space.
 *
 * @param url - The URL of the rainlang file to fetch
 * @returns A Promise that resolves to an array of objects containing file names and their corresponding URLs
 * @throws Will throw an error if the fetch fails, if the response is not ok, or if the rainlang format is invalid
 *
 * @example
 * const files = await fetchParseRainlangFile('https://example.com/rainlang');
 * // Returns: [{ name: 'file1', url: 'https://example.com/file1.rain' }, ...]
 */

export const fetchParseRainlang = async (url: string): Promise<{ name: string; url: string }[]> => {
	try {
		const response = await fetch(url);
		if (!response.ok) {
			throw new Error('Failed to fetch rainlang.');
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
			throw new Error('Invalid stategy rainlang.');
		}
		return files;
	} catch (e) {
		throw new Error(e instanceof Error ? e.message : 'Unknown error.');
	}
};

export const fetchRainlangDotrains = async (url: string): Promise<RainlangDotrain[]> => {
	const files = await fetchParseRainlang(url);
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
	rainlangDotrains: RainlangDotrain[]
): Promise<OrderValidationResult> {
	const ordersPromises = rainlangDotrains.map(async (rainlangDotrain) => {
		try {
			const result = await DotrainOrderGui.getOrderDetails(rainlangDotrain.dotrain);

			if (result.error) {
				throw new Error(result.error.msg);
			}

			return {
				valid: true,
				data: {
					...rainlangDotrain,
					details: result.value
				}
			};
		} catch (error) {
			return {
				valid: false,
				data: {
					name: rainlangDotrain.name,
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
