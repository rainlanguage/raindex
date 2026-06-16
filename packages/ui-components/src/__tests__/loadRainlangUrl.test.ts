import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { Mock } from 'vitest';
import { loadRainlangUrl } from '../lib/services/loadRainlangUrl';
import { RainlangManager } from '../lib/providers/rainlang/RainlangManager';
import { initialRainlang } from '../__fixtures__/RainlangManager';
import { DotrainRainlang } from '@rainlanguage/orderbook';

// Mock dependencies
vi.mock('@rainlanguage/orderbook', () => ({
	DotrainRainlang: {
		validate: vi.fn()
	}
}));

describe('loadRainlangUrl', () => {
	beforeEach(() => {
		vi.resetAllMocks();
		const originalLocation = window.location;
		const mockLocation = { ...originalLocation, reload: vi.fn() };
		Object.defineProperty(window, 'location', {
			writable: true,
			value: mockLocation
		});
	});

	it('should throw an error if no URL is provided', async () => {
		const mockRainlangManager = initialRainlang as RainlangManager;
		await expect(loadRainlangUrl('', mockRainlangManager)).rejects.toThrow('No URL provided');
	});

	it('should throw an error if no rainlang manager is provided', async () => {
		await expect(
			loadRainlangUrl('https://example.com/rainlang', null as unknown as RainlangManager)
		).rejects.toThrow('Rainlang manager is required');
	});

	it('should successfully load rainlang URL and reload the page', async () => {
		const testUrl = 'https://example.com/rainlang';
		const mockRainlangManager = initialRainlang as RainlangManager;

		(DotrainRainlang.validate as Mock).mockResolvedValueOnce({ value: {} });
		await loadRainlangUrl(testUrl, mockRainlangManager);
		expect(DotrainRainlang.validate).toHaveBeenCalledWith(testUrl);
		expect(mockRainlangManager.setRainlang).toHaveBeenCalledWith(testUrl);
		expect(window.location.reload).toHaveBeenCalled();
	});

	it('should throw an error if fetching rainlang fails', async () => {
		const testUrl = 'https://example.com/rainlang';
		const errorMessage = 'Fetch failed';
		const mockRainlangManager = {
			setRainlang: vi.fn()
		} as unknown as RainlangManager;

		(DotrainRainlang.validate as Mock).mockRejectedValueOnce(new Error(errorMessage));

		await expect(loadRainlangUrl(testUrl, mockRainlangManager)).rejects.toThrow(errorMessage);

		expect(mockRainlangManager.setRainlang).not.toHaveBeenCalled();
		expect(window.location.reload).not.toHaveBeenCalled();
	});

	it('should handle non-Error exception during rainlang fetch', async () => {
		const testUrl = 'https://example.com/rainlang';
		const mockRainlangManager = {
			setRainlang: vi.fn()
		} as unknown as RainlangManager;

		(DotrainRainlang.validate as Mock).mockRejectedValueOnce('String error');

		await expect(loadRainlangUrl(testUrl, mockRainlangManager)).rejects.toThrow(
			'Failed to update rainlang URL'
		);

		expect(mockRainlangManager.setRainlang).not.toHaveBeenCalled();
		expect(window.location.reload).not.toHaveBeenCalled();
	});
});
