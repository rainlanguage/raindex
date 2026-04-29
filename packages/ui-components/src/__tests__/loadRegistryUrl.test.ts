import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { Mock } from 'vitest';
import { loadRegistryUrl } from '../lib/services/loadRegistryUrl';
import { RegistryManager } from '../lib/providers/registry/RegistryManager';
import { initialRegistry } from '../__fixtures__/RegistryManager';
import { DotrainRegistry } from '@rainlanguage/raindex';

// Mock dependencies
vi.mock('@rainlanguage/raindex', () => ({
	DotrainRegistry: {
		validate: vi.fn()
	}
}));

describe('loadRegistryUrl', () => {
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
		const mockRegistryManager = initialRegistry as RegistryManager;
		await expect(loadRegistryUrl('', mockRegistryManager)).rejects.toThrow('No URL provided');
	});

	it('should throw an error if no registry manager is provided', async () => {
		await expect(
			loadRegistryUrl('https://example.com/registry', null as unknown as RegistryManager)
		).rejects.toThrow('Registry manager is required');
	});

	it('should successfully load registry URL and reload the page', async () => {
		const testUrl = 'https://example.com/registry';
		const mockRegistryManager = initialRegistry as RegistryManager;

		(DotrainRegistry.validate as Mock).mockResolvedValueOnce({ value: {} });
		await loadRegistryUrl(testUrl, mockRegistryManager);
		expect(DotrainRegistry.validate).toHaveBeenCalledWith(testUrl);
		expect(mockRegistryManager.setRegistry).toHaveBeenCalledWith(testUrl);
		expect(window.location.reload).toHaveBeenCalled();
	});

	it('should throw an error if fetching registry fails', async () => {
		const testUrl = 'https://example.com/registry';
		const errorMessage = 'Fetch failed';
		const mockRegistryManager = {
			setRegistry: vi.fn()
		} as unknown as RegistryManager;

		(DotrainRegistry.validate as Mock).mockRejectedValueOnce(new Error(errorMessage));

		await expect(loadRegistryUrl(testUrl, mockRegistryManager)).rejects.toThrow(errorMessage);

		expect(mockRegistryManager.setRegistry).not.toHaveBeenCalled();
		expect(window.location.reload).not.toHaveBeenCalled();
	});

	it('should handle non-Error exception during registry fetch', async () => {
		const testUrl = 'https://example.com/registry';
		const mockRegistryManager = {
			setRegistry: vi.fn()
		} as unknown as RegistryManager;

		(DotrainRegistry.validate as Mock).mockRejectedValueOnce('String error');

		await expect(loadRegistryUrl(testUrl, mockRegistryManager)).rejects.toThrow(
			'Failed to update registry URL'
		);

		expect(mockRegistryManager.setRegistry).not.toHaveBeenCalled();
		expect(window.location.reload).not.toHaveBeenCalled();
	});
});
