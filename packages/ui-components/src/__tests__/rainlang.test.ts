import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
	fetchParseRainlang,
	fetchRainlangDotrains,
	validateOrders
} from '../lib/services/rainlang';
import { DotrainOrderGui } from '@rainlanguage/orderbook';
import type { Mock } from 'vitest';

// Mock the DotrainOrderGui dependency
vi.mock('@rainlanguage/orderbook', () => ({
	DotrainOrderGui: {
		getOrderDetails: vi.fn()
	}
}));

describe('fetchParseRainlang', () => {
	it('should parse rainlang file content correctly', async () => {
		const mockResponse = `file1.js https://example.com/file1.js
file2.js https://example.com/file2.js`;

		global.fetch = vi.fn().mockResolvedValue({
			ok: true,
			text: () => Promise.resolve(mockResponse)
		});

		const result = await fetchParseRainlang('https://example.com/rainlang');
		expect(result).toEqual([
			{ name: 'file1.js', url: 'https://example.com/file1.js' },
			{ name: 'file2.js', url: 'https://example.com/file2.js' }
		]);
	});

	it('should handle failed fetch response', async () => {
		global.fetch = vi.fn().mockResolvedValue({
			ok: false
		});

		await expect(fetchParseRainlang('https://example.com/rainlang')).rejects.toThrow(
			'Failed to fetch rainlang'
		);
	});

	it('should handle network errors', async () => {
		global.fetch = vi.fn().mockRejectedValue(new Error('Network error'));

		await expect(fetchParseRainlang('https://example.com/rainlang')).rejects.toThrow(
			'Network error'
		);
	});
});

describe('fetchRainlangDotrains', () => {
	it('should fetch and parse dotrains correctly', async () => {
		const mockRainlang = `file1.rain https://example.com/file1.rain
file2.rain https://example.com/file2.rain`;

		const mockDotrain1 = 'content of file1';
		const mockDotrain2 = 'content of file2';

		global.fetch = vi
			.fn()
			.mockResolvedValueOnce({
				ok: true,
				text: () => Promise.resolve(mockRainlang)
			})
			.mockResolvedValueOnce({
				ok: true,
				text: () => Promise.resolve(mockDotrain1)
			})
			.mockResolvedValueOnce({
				ok: true,
				text: () => Promise.resolve(mockDotrain2)
			});

		const result = await fetchRainlangDotrains('https://example.com/rainlang');
		expect(result).toEqual([
			{ name: 'file1.rain', dotrain: mockDotrain1 },
			{ name: 'file2.rain', dotrain: mockDotrain2 }
		]);
	});

	it('should handle failed dotrain fetch', async () => {
		const mockRainlang = `file1.rain https://example.com/file1.rain`;

		global.fetch = vi
			.fn()
			.mockResolvedValueOnce({
				ok: true,
				text: () => Promise.resolve(mockRainlang)
			})
			.mockResolvedValueOnce({
				ok: false
			});

		await expect(fetchRainlangDotrains('https://example.com/rainlang')).rejects.toThrow(
			'Failed to fetch dotrain for file1.rain'
		);
	});

	it('should handle network errors during dotrain fetch', async () => {
		const mockRainlang = `file1.rain https://example.com/file1.rain`;

		global.fetch = vi
			.fn()
			.mockResolvedValueOnce({
				ok: true,
				text: () => Promise.resolve(mockRainlang)
			})
			.mockRejectedValueOnce(new Error('Network error'));

		await expect(fetchRainlangDotrains('https://example.com/rainlang')).rejects.toThrow(
			'Error fetching dotrain for file1.rain: Network error'
		);
	});
});

describe('validateOrders', () => {
	beforeEach(() => {
		vi.resetAllMocks();
	});

	it('should validate orders and categorize them properly', async () => {
		// Input data
		const rainlangDotrains = [
			{ name: 'valid.rain', dotrain: 'valid dotrain content' },
			{ name: 'invalid.rain', dotrain: 'invalid dotrain content' },
			{ name: 'another-valid.rain', dotrain: 'another valid content' }
		];

		// Set up mock responses for the DotrainOrderGui
		(DotrainOrderGui.getOrderDetails as Mock)
			.mockResolvedValueOnce({
				value: { name: 'Valid Order', description: 'A valid order' },
				error: null
			})
			.mockResolvedValueOnce({
				error: { msg: 'Invalid syntax' },
				value: null
			})
			.mockResolvedValueOnce({
				value: { name: 'Another Valid', description: 'Another valid order' },
				error: null
			});

		// Call the function with our test data
		const result = await validateOrders(rainlangDotrains);

		// Verify DotrainOrderGui was called correctly
		expect(DotrainOrderGui.getOrderDetails).toHaveBeenCalledTimes(3);
		expect(DotrainOrderGui.getOrderDetails).toHaveBeenCalledWith('valid dotrain content');
		expect(DotrainOrderGui.getOrderDetails).toHaveBeenCalledWith('invalid dotrain content');
		expect(DotrainOrderGui.getOrderDetails).toHaveBeenCalledWith('another valid content');

		// Verify the valid orders are processed correctly
		expect(result.validOrders).toHaveLength(2);
		expect(result.validOrders[0].name).toBe('valid.rain');
		expect(result.validOrders[0].dotrain).toBe('valid dotrain content');
		expect(result.validOrders[0].details).toEqual({
			name: 'Valid Order',
			description: 'A valid order'
		});

		// Verify the invalid orders are processed correctly
		expect(result.invalidOrders).toHaveLength(1);
		expect(result.invalidOrders[0].name).toBe('invalid.rain');
		expect(result.invalidOrders[0].error).toBe('Invalid syntax');
	});

	it('should handle exceptions thrown during order validation', async () => {
		// Input data
		const rainlangDotrains = [{ name: 'error.rain', dotrain: 'will throw error' }];

		// Mock the DotrainOrderGui to throw an exception
		(DotrainOrderGui.getOrderDetails as Mock).mockRejectedValueOnce(
			new Error('Unexpected parsing error')
		);

		// Call the function
		const result = await validateOrders(rainlangDotrains);

		// Verify results
		expect(result.validOrders).toHaveLength(0);
		expect(result.invalidOrders).toHaveLength(1);
		expect(result.invalidOrders[0].name).toBe('error.rain');
		expect(result.invalidOrders[0].error).toBe('Unexpected parsing error');
	});

	it('should handle non-Error objects being thrown', async () => {
		// Input data
		const rainlangDotrains = [{ name: 'string-error.rain', dotrain: 'will throw string' }];

		// Mock the DotrainOrderGui to throw a string instead of an Error
		(DotrainOrderGui.getOrderDetails as Mock).mockRejectedValueOnce('String error message');

		// Call the function
		const result = await validateOrders(rainlangDotrains);

		// Verify results
		expect(result.validOrders).toHaveLength(0);
		expect(result.invalidOrders).toHaveLength(1);
		expect(result.invalidOrders[0].name).toBe('string-error.rain');
		expect(result.invalidOrders[0].error).toBe('String error message');
	});

	it('should process an empty array of orders', async () => {
		const result = await validateOrders([]);

		expect(result.validOrders).toEqual([]);
		expect(result.invalidOrders).toEqual([]);
		expect(DotrainOrderGui.getOrderDetails).not.toHaveBeenCalled();
	});

	it('should handle mixed validation results correctly', async () => {
		// Create a mix of scenarios
		const rainlangDotrains = [
			{ name: 'valid1.rain', dotrain: 'valid content 1' },
			{ name: 'error.rain', dotrain: 'will throw error' },
			{ name: 'valid2.rain', dotrain: 'valid content 2' },
			{ name: 'invalid.rain', dotrain: 'invalid content' }
		];

		// Set up mock responses
		(DotrainOrderGui.getOrderDetails as Mock)
			.mockResolvedValueOnce({
				value: { orderName: 'Order 1', description: 'Description 1' },
				error: null
			})
			.mockRejectedValueOnce(new Error('Processing error'))
			.mockResolvedValueOnce({
				value: { orderName: 'Order 2', description: 'Description 2' },
				error: null
			})
			.mockResolvedValueOnce({
				error: { msg: 'Validation failed' },
				value: null
			});

		// Call the function
		const result = await validateOrders(rainlangDotrains);

		// Verify results
		expect(result.validOrders).toHaveLength(2);
		expect(result.validOrders[0].name).toBe('valid1.rain');
		expect(result.validOrders[1].name).toBe('valid2.rain');

		expect(result.invalidOrders).toHaveLength(2);
		expect(result.invalidOrders[0].name).toBe('error.rain');
		expect(result.invalidOrders[0].error).toBe('Processing error');
		expect(result.invalidOrders[1].name).toBe('invalid.rain');
		expect(result.invalidOrders[1].error).toBe('Validation failed');
	});
});
