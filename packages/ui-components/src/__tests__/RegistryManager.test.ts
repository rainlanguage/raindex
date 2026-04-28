import { describe, it, expect, vi, beforeEach } from 'vitest';
import { RegistryManager } from '../lib/providers/registry/RegistryManager';

const DEFAULT_REGISTRY_URL = 'https://default.registry.url/registry.json';
const CUSTOM_REGISTRY_URL = 'https://custom.registry.url/registry.json';
const STORAGE_KEY = 'registry';

const mockLocalStorage = (() => {
	let store: Record<string, string> = {};
	return {
		getItem: vi.fn((key: string) => store[key] || null),
		setItem: vi.fn((key: string, value: string) => {
			store[key] = value;
		}),
		removeItem: vi.fn((key: string) => {
			delete store[key];
		}),
		clear: () => {
			store = {};
		},
		getStore: () => store
	};
})();

const mockLocation = (searchParams: URLSearchParams) => ({
	href: `http://localhost/?${searchParams.toString()}`,
	searchParams
});

const mockHistory = {
	pushState: vi.fn()
};

vi.stubGlobal('localStorage', mockLocalStorage);
vi.stubGlobal('history', mockHistory);

const setMockLocation = (params: Record<string, string>) => {
	const searchParams = new URLSearchParams(params);
	Object.defineProperty(window, 'location', {
		value: mockLocation(searchParams),
		writable: true
	});
};

describe('RegistryManager', () => {
	beforeEach(() => {
		mockLocalStorage.clear();
		vi.clearAllMocks();
		setMockLocation({});
	});

	it('should initialize with default registry if no URL param or localStorage', () => {
		const manager = new RegistryManager(DEFAULT_REGISTRY_URL);
		expect(manager.getCurrentRegistry()).toBe(DEFAULT_REGISTRY_URL);
		expect(mockLocalStorage.getItem).toHaveBeenCalledWith(STORAGE_KEY);
		expect(mockLocalStorage.setItem).not.toHaveBeenCalled();
	});

	it('should initialize with URL parameter if present', () => {
		setMockLocation({ [STORAGE_KEY]: CUSTOM_REGISTRY_URL });
		const manager = new RegistryManager(DEFAULT_REGISTRY_URL);
		expect(manager.getCurrentRegistry()).toBe(CUSTOM_REGISTRY_URL);
		expect(mockLocalStorage.setItem).toHaveBeenCalledWith(STORAGE_KEY, CUSTOM_REGISTRY_URL);
	});

	it('should initialize with localStorage value if present (and no URL param)', () => {
		mockLocalStorage.setItem(STORAGE_KEY, CUSTOM_REGISTRY_URL);
		const manager = new RegistryManager(DEFAULT_REGISTRY_URL);
		expect(manager.getCurrentRegistry()).toBe(CUSTOM_REGISTRY_URL);
		expect(mockLocalStorage.getItem).toHaveBeenCalledWith(STORAGE_KEY);
		expect(mockLocalStorage.setItem).toHaveBeenCalledTimes(1);
	});

	it('should prioritize URL parameter over localStorage on initialization', () => {
		const urlRegistry = 'https://from.url/registry.json';
		setMockLocation({ [STORAGE_KEY]: urlRegistry });
		mockLocalStorage.setItem(STORAGE_KEY, CUSTOM_REGISTRY_URL);

		const manager = new RegistryManager(DEFAULT_REGISTRY_URL);
		expect(manager.getCurrentRegistry()).toBe(urlRegistry);
		expect(mockLocalStorage.setItem).toHaveBeenCalledWith(STORAGE_KEY, urlRegistry);
	});

	it('getCurrentRegistry() should return the current registry', () => {
		const manager = new RegistryManager(DEFAULT_REGISTRY_URL);
		expect(manager.getCurrentRegistry()).toBe(DEFAULT_REGISTRY_URL);

		setMockLocation({ [STORAGE_KEY]: CUSTOM_REGISTRY_URL });
		const manager2 = new RegistryManager(DEFAULT_REGISTRY_URL);
		expect(manager2.getCurrentRegistry()).toBe(CUSTOM_REGISTRY_URL);
	});

	it('setRegistry() should update current registry, localStorage, and URL', () => {
		const manager = new RegistryManager(DEFAULT_REGISTRY_URL);
		manager.setRegistry(CUSTOM_REGISTRY_URL);

		expect(manager.getCurrentRegistry()).toBe(CUSTOM_REGISTRY_URL);
		expect(mockLocalStorage.setItem).toHaveBeenCalledWith(STORAGE_KEY, CUSTOM_REGISTRY_URL);
		expect(mockHistory.pushState).toHaveBeenCalledTimes(1);
		const expectedUrl = new URL(window.location.href);
		expectedUrl.searchParams.set(STORAGE_KEY, CUSTOM_REGISTRY_URL);
		expect(mockHistory.pushState).toHaveBeenCalledWith({}, '', expectedUrl.toString());
	});

	it('resetToDefault() should reset registry, clear localStorage, and update URL', () => {
		mockLocalStorage.setItem(STORAGE_KEY, CUSTOM_REGISTRY_URL);
		setMockLocation({ [STORAGE_KEY]: CUSTOM_REGISTRY_URL });
		const manager = new RegistryManager(DEFAULT_REGISTRY_URL);
		expect(manager.getCurrentRegistry()).toBe(CUSTOM_REGISTRY_URL);

		manager.resetToDefault();

		expect(manager.getCurrentRegistry()).toBe(DEFAULT_REGISTRY_URL);
		expect(mockLocalStorage.removeItem).toHaveBeenCalledWith(STORAGE_KEY);
		expect(mockHistory.pushState).toHaveBeenCalledTimes(1);
		const expectedUrl = new URL(window.location.href);
		expectedUrl.searchParams.delete(STORAGE_KEY);
		expect(mockHistory.pushState).toHaveBeenCalledWith({}, '', expectedUrl.toString());
	});

	it('updateUrlWithRegistry() should update URL search parameter', () => {
		const manager = new RegistryManager(DEFAULT_REGISTRY_URL);
		manager.updateUrlWithRegistry(CUSTOM_REGISTRY_URL);

		expect(mockHistory.pushState).toHaveBeenCalledTimes(1);
		const expectedUrl = new URL(window.location.href);
		expectedUrl.searchParams.set(STORAGE_KEY, CUSTOM_REGISTRY_URL);
		expect(mockHistory.pushState).toHaveBeenCalledWith({}, '', expectedUrl.toString());
	});

	it('updateUrlWithRegistry() should remove URL search parameter when value is null', () => {
		setMockLocation({ [STORAGE_KEY]: CUSTOM_REGISTRY_URL });
		const manager = new RegistryManager(DEFAULT_REGISTRY_URL);
		manager.updateUrlWithRegistry(null);

		expect(mockHistory.pushState).toHaveBeenCalledTimes(1);
		const expectedUrl = new URL(window.location.href);
		expectedUrl.searchParams.delete(STORAGE_KEY);
		expect(mockHistory.pushState).toHaveBeenCalledWith({}, '', expectedUrl.toString());
	});

	it('isCustomRegistry() should return false when using default registry', () => {
		const manager = new RegistryManager(DEFAULT_REGISTRY_URL);
		expect(manager.isCustomRegistry()).toBe(false);
	});

	it('isCustomRegistry() should return true when using a custom registry', () => {
		mockLocalStorage.setItem(STORAGE_KEY, CUSTOM_REGISTRY_URL);
		const manager = new RegistryManager(DEFAULT_REGISTRY_URL);
		expect(manager.isCustomRegistry()).toBe(true);
	});

	it('isCustomRegistry() should return false after resetting to default', () => {
		mockLocalStorage.setItem(STORAGE_KEY, CUSTOM_REGISTRY_URL);
		const manager = new RegistryManager(DEFAULT_REGISTRY_URL);
		expect(manager.isCustomRegistry()).toBe(true);
		manager.resetToDefault();
		expect(manager.isCustomRegistry()).toBe(false);
	});

	it('should handle localStorage errors gracefully (at least not crash)', () => {
		vi.spyOn(mockLocalStorage, 'getItem').mockImplementation(() => {
			throw new Error('localStorage unavailable');
		});
		vi.spyOn(mockLocalStorage, 'setItem').mockImplementation(() => {
			throw new Error('localStorage unavailable');
		});
		vi.spyOn(mockLocalStorage, 'removeItem').mockImplementation(() => {
			throw new Error('localStorage unavailable');
		});

		expect(() => new RegistryManager(DEFAULT_REGISTRY_URL)).toThrow(
			/Failed to access localStorage|Failed to save to localStorage/
		);

		vi.restoreAllMocks();
	});
});
