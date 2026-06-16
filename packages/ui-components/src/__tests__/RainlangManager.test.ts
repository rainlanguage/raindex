import { describe, it, expect, vi, beforeEach } from 'vitest';
import { RainlangManager } from '../lib/providers/rainlang/RainlangManager';

const DEFAULT_RAINLANG_URL = 'https://default.rainlang.url/rainlang.json';
const CUSTOM_RAINLANG_URL = 'https://custom.rainlang.url/rainlang.json';
const STORAGE_KEY = 'rainlang';

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

describe('RainlangManager', () => {
	beforeEach(() => {
		mockLocalStorage.clear();
		vi.clearAllMocks();
		setMockLocation({});
	});

	it('should initialize with default rainlang if no URL param or localStorage', () => {
		const manager = new RainlangManager(DEFAULT_RAINLANG_URL);
		expect(manager.getCurrentRainlang()).toBe(DEFAULT_RAINLANG_URL);
		expect(mockLocalStorage.getItem).toHaveBeenCalledWith(STORAGE_KEY);
		expect(mockLocalStorage.setItem).not.toHaveBeenCalled();
	});

	it('should initialize with URL parameter if present', () => {
		setMockLocation({ [STORAGE_KEY]: CUSTOM_RAINLANG_URL });
		const manager = new RainlangManager(DEFAULT_RAINLANG_URL);
		expect(manager.getCurrentRainlang()).toBe(CUSTOM_RAINLANG_URL);
		expect(mockLocalStorage.setItem).toHaveBeenCalledWith(STORAGE_KEY, CUSTOM_RAINLANG_URL);
	});

	it('should initialize with localStorage value if present (and no URL param)', () => {
		mockLocalStorage.setItem(STORAGE_KEY, CUSTOM_RAINLANG_URL);
		const manager = new RainlangManager(DEFAULT_RAINLANG_URL);
		expect(manager.getCurrentRainlang()).toBe(CUSTOM_RAINLANG_URL);
		expect(mockLocalStorage.getItem).toHaveBeenCalledWith(STORAGE_KEY);
		expect(mockLocalStorage.setItem).toHaveBeenCalledTimes(1);
	});

	it('should prioritize URL parameter over localStorage on initialization', () => {
		const urlRainlang = 'https://from.url/rainlang.json';
		setMockLocation({ [STORAGE_KEY]: urlRainlang });
		mockLocalStorage.setItem(STORAGE_KEY, CUSTOM_RAINLANG_URL);

		const manager = new RainlangManager(DEFAULT_RAINLANG_URL);
		expect(manager.getCurrentRainlang()).toBe(urlRainlang);
		expect(mockLocalStorage.setItem).toHaveBeenCalledWith(STORAGE_KEY, urlRainlang);
	});

	it('getCurrentRainlang() should return the current rainlang', () => {
		const manager = new RainlangManager(DEFAULT_RAINLANG_URL);
		expect(manager.getCurrentRainlang()).toBe(DEFAULT_RAINLANG_URL);

		setMockLocation({ [STORAGE_KEY]: CUSTOM_RAINLANG_URL });
		const manager2 = new RainlangManager(DEFAULT_RAINLANG_URL);
		expect(manager2.getCurrentRainlang()).toBe(CUSTOM_RAINLANG_URL);
	});

	it('setRainlang() should update current rainlang, localStorage, and URL', () => {
		const manager = new RainlangManager(DEFAULT_RAINLANG_URL);
		manager.setRainlang(CUSTOM_RAINLANG_URL);

		expect(manager.getCurrentRainlang()).toBe(CUSTOM_RAINLANG_URL);
		expect(mockLocalStorage.setItem).toHaveBeenCalledWith(STORAGE_KEY, CUSTOM_RAINLANG_URL);
		expect(mockHistory.pushState).toHaveBeenCalledTimes(1);
		const expectedUrl = new URL(window.location.href);
		expectedUrl.searchParams.set(STORAGE_KEY, CUSTOM_RAINLANG_URL);
		expect(mockHistory.pushState).toHaveBeenCalledWith({}, '', expectedUrl.toString());
	});

	it('resetToDefault() should reset rainlang, clear localStorage, and update URL', () => {
		mockLocalStorage.setItem(STORAGE_KEY, CUSTOM_RAINLANG_URL);
		setMockLocation({ [STORAGE_KEY]: CUSTOM_RAINLANG_URL });
		const manager = new RainlangManager(DEFAULT_RAINLANG_URL);
		expect(manager.getCurrentRainlang()).toBe(CUSTOM_RAINLANG_URL);

		manager.resetToDefault();

		expect(manager.getCurrentRainlang()).toBe(DEFAULT_RAINLANG_URL);
		expect(mockLocalStorage.removeItem).toHaveBeenCalledWith(STORAGE_KEY);
		expect(mockHistory.pushState).toHaveBeenCalledTimes(1);
		const expectedUrl = new URL(window.location.href);
		expectedUrl.searchParams.delete(STORAGE_KEY);
		expect(mockHistory.pushState).toHaveBeenCalledWith({}, '', expectedUrl.toString());
	});

	it('updateUrlWithRainlang() should update URL search parameter', () => {
		const manager = new RainlangManager(DEFAULT_RAINLANG_URL);
		manager.updateUrlWithRainlang(CUSTOM_RAINLANG_URL);

		expect(mockHistory.pushState).toHaveBeenCalledTimes(1);
		const expectedUrl = new URL(window.location.href);
		expectedUrl.searchParams.set(STORAGE_KEY, CUSTOM_RAINLANG_URL);
		expect(mockHistory.pushState).toHaveBeenCalledWith({}, '', expectedUrl.toString());
	});

	it('updateUrlWithRainlang() should remove URL search parameter when value is null', () => {
		setMockLocation({ [STORAGE_KEY]: CUSTOM_RAINLANG_URL });
		const manager = new RainlangManager(DEFAULT_RAINLANG_URL);
		manager.updateUrlWithRainlang(null);

		expect(mockHistory.pushState).toHaveBeenCalledTimes(1);
		const expectedUrl = new URL(window.location.href);
		expectedUrl.searchParams.delete(STORAGE_KEY);
		expect(mockHistory.pushState).toHaveBeenCalledWith({}, '', expectedUrl.toString());
	});

	it('isCustomRainlang() should return false when using default rainlang', () => {
		const manager = new RainlangManager(DEFAULT_RAINLANG_URL);
		expect(manager.isCustomRainlang()).toBe(false);
	});

	it('isCustomRainlang() should return true when using a custom rainlang', () => {
		mockLocalStorage.setItem(STORAGE_KEY, CUSTOM_RAINLANG_URL);
		const manager = new RainlangManager(DEFAULT_RAINLANG_URL);
		expect(manager.isCustomRainlang()).toBe(true);
	});

	it('isCustomRainlang() should return false after resetting to default', () => {
		mockLocalStorage.setItem(STORAGE_KEY, CUSTOM_RAINLANG_URL);
		const manager = new RainlangManager(DEFAULT_RAINLANG_URL);
		expect(manager.isCustomRainlang()).toBe(true);
		manager.resetToDefault();
		expect(manager.isCustomRainlang()).toBe(false);
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

		expect(() => new RainlangManager(DEFAULT_RAINLANG_URL)).toThrow(
			/Failed to access localStorage|Failed to save to localStorage/
		);

		vi.restoreAllMocks();
	});
});
