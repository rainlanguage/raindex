/**
 * Manages rainlang URL settings, persisting values in localStorage and URL parameters
 */
export class RainlangManager {
	/** The default rainlang URL to fall back to */
	private defaultRainlang: string;

	/** The currently selected rainlang URL */
	private currentRainlang: string | null;

	/** Key used for localStorage and URL parameters */
	private static STORAGE_KEY = 'rainlang';

	/**
	 * Create a new RainlangManager
	 * @param defaultRainlang The default rainlang URL to use.
	 */
	constructor(defaultRainlang: string) {
		this.defaultRainlang = defaultRainlang;
		this.currentRainlang = this.loadRainlangFromStorageOrUrl();
	}

	/**
	 * Initialize rainlang from URL param or local storage
	 * @returns The rainlang URL to use
	 */
	private loadRainlangFromStorageOrUrl(): string {
		const urlParam = this.getRainlangParamFromUrl();
		if (urlParam) {
			this.setRainlangToLocalStorage(urlParam);
			return urlParam;
		}
		return this.getRainlangFromLocalStorage() ?? this.defaultRainlang;
	}

	/**
	 * Get the rainlang from the URL param
	 * @returns The rainlang value from URL or null if not present
	 * @throws Error if URL parsing fails
	 */
	private getRainlangParamFromUrl(): string | null {
		try {
			return new URL(window.location.href).searchParams.get(RainlangManager.STORAGE_KEY);
		} catch (error) {
			throw new Error(
				'Failed to get rainlang parameter: ' +
					(error instanceof Error ? error.message : String(error))
			);
		}
	}

	/**
	 * Save the rainlang to local storage
	 * @param rainlang The rainlang URL to save
	 * @throws Error if localStorage is not available
	 */
	private setRainlangToLocalStorage(rainlang: string): void {
		try {
			localStorage.setItem(RainlangManager.STORAGE_KEY, rainlang);
		} catch (error) {
			throw new Error(
				'Failed to save to localStorage: ' +
					(error instanceof Error ? error.message : String(error))
			);
		}
	}

	/**
	 * Retrieve the rainlang from local storage
	 * @returns The stored rainlang URL or null if not found
	 * @throws Error if localStorage is not available
	 */
	private getRainlangFromLocalStorage(): string | null {
		try {
			return localStorage.getItem(RainlangManager.STORAGE_KEY);
		} catch (error) {
			throw new Error(
				'Failed to access localStorage: ' + (error instanceof Error ? error.message : String(error))
			);
		}
	}

	/**
	 * Get the currently active rainlang
	 * @returns The current rainlang URL, falling back to default if not set
	 */
	public getCurrentRainlang(): string {
		return this.currentRainlang ?? this.defaultRainlang;
	}

	/**
	 * Set the rainlang and update both localStorage and URL
	 * @param rainlang The new rainlang URL to set
	 */
	public setRainlang(rainlang: string): void {
		this.currentRainlang = rainlang;
		this.setRainlangToLocalStorage(rainlang);
		this.updateUrlWithRainlang();
	}

	/**
	 * Reset to the default rainlang, clearing both localStorage and URL param
	 * @throws Error if localStorage is not available
	 */
	public resetToDefault(): void {
		this.currentRainlang = this.defaultRainlang;
		try {
			localStorage.removeItem(RainlangManager.STORAGE_KEY);
		} catch (error) {
			throw new Error(
				'Failed to clear rainlang from localStorage: ' +
					(error instanceof Error ? error.message : String(error))
			);
		}
		this.updateUrlWithRainlang(null);
	}

	/**
	 * Update the URL param to reflect the current or specified rainlang
	 * @param value The rainlang value to set in URL, defaults to current rainlang
	 * @throws Error if URL manipulation fails
	 */
	public updateUrlWithRainlang(value: string | null = this.currentRainlang): void {
		try {
			const url = new URL(window.location.href);
			if (value) {
				url.searchParams.set(RainlangManager.STORAGE_KEY, value);
			} else {
				url.searchParams.delete(RainlangManager.STORAGE_KEY);
			}
			window.history.pushState({}, '', url.toString());
		} catch (error) {
			throw new Error(
				'Failed to update URL parameter: ' +
					(error instanceof Error ? error.message : String(error))
			);
		}
	}

	/**
	 * Check if the current rainlang is custom (different from the default)
	 * @returns True if using a non-default rainlang
	 */
	public isCustomRainlang(): boolean {
		return (
			this.currentRainlang !== undefined &&
			this.currentRainlang !== null &&
			this.currentRainlang !== this.defaultRainlang
		);
	}
}
