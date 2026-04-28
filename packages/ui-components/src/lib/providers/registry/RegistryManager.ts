/**
 * Manages registry URL settings, persisting values in localStorage and URL parameters
 */
export class RegistryManager {
	/** The default registry URL to fall back to */
	private defaultRegistry: string;

	/** The currently selected registry URL */
	private currentRegistry: string | null;

	/** Key used for localStorage and URL parameters */
	private static STORAGE_KEY = 'registry';

	/**
	 * Create a new RegistryManager
	 * @param defaultRegistry The default registry URL to use.
	 */
	constructor(defaultRegistry: string) {
		this.defaultRegistry = defaultRegistry;
		this.currentRegistry = this.loadRegistryFromStorageOrUrl();
	}

	/**
	 * Initialize registry from URL param or local storage
	 * @returns The registry URL to use
	 */
	private loadRegistryFromStorageOrUrl(): string {
		const urlParam = this.getRegistryParamFromUrl();
		if (urlParam) {
			this.setRegistryToLocalStorage(urlParam);
			return urlParam;
		}
		return this.getRegistryFromLocalStorage() ?? this.defaultRegistry;
	}

	/**
	 * Get the registry from the URL param
	 * @returns The registry value from URL or null if not present
	 * @throws Error if URL parsing fails
	 */
	private getRegistryParamFromUrl(): string | null {
		try {
			return new URL(window.location.href).searchParams.get(RegistryManager.STORAGE_KEY);
		} catch (error) {
			throw new Error(
				'Failed to get registry parameter: ' +
					(error instanceof Error ? error.message : String(error))
			);
		}
	}

	/**
	 * Save the registry to local storage
	 * @param registry The registry URL to save
	 * @throws Error if localStorage is not available
	 */
	private setRegistryToLocalStorage(registry: string): void {
		try {
			localStorage.setItem(RegistryManager.STORAGE_KEY, registry);
		} catch (error) {
			throw new Error(
				'Failed to save to localStorage: ' +
					(error instanceof Error ? error.message : String(error))
			);
		}
	}

	/**
	 * Retrieve the registry from local storage
	 * @returns The stored registry URL or null if not found
	 * @throws Error if localStorage is not available
	 */
	private getRegistryFromLocalStorage(): string | null {
		try {
			return localStorage.getItem(RegistryManager.STORAGE_KEY);
		} catch (error) {
			throw new Error(
				'Failed to access localStorage: ' + (error instanceof Error ? error.message : String(error))
			);
		}
	}

	/**
	 * Get the currently active registry
	 * @returns The current registry URL, falling back to default if not set
	 */
	public getCurrentRegistry(): string {
		return this.currentRegistry ?? this.defaultRegistry;
	}

	/**
	 * Set the registry and update both localStorage and URL
	 * @param registry The new registry URL to set
	 */
	public setRegistry(registry: string): void {
		this.currentRegistry = registry;
		this.setRegistryToLocalStorage(registry);
		this.updateUrlWithRegistry();
	}

	/**
	 * Reset to the default registry, clearing both localStorage and URL param
	 * @throws Error if localStorage is not available
	 */
	public resetToDefault(): void {
		this.currentRegistry = this.defaultRegistry;
		try {
			localStorage.removeItem(RegistryManager.STORAGE_KEY);
		} catch (error) {
			throw new Error(
				'Failed to clear registry from localStorage: ' +
					(error instanceof Error ? error.message : String(error))
			);
		}
		this.updateUrlWithRegistry(null);
	}

	/**
	 * Update the URL param to reflect the current or specified registry
	 * @param value The registry value to set in URL, defaults to current registry
	 * @throws Error if URL manipulation fails
	 */
	public updateUrlWithRegistry(value: string | null = this.currentRegistry): void {
		try {
			const url = new URL(window.location.href);
			if (value) {
				url.searchParams.set(RegistryManager.STORAGE_KEY, value);
			} else {
				url.searchParams.delete(RegistryManager.STORAGE_KEY);
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
	 * Check if the current registry is custom (different from the default)
	 * @returns True if using a non-default registry
	 */
	public isCustomRegistry(): boolean {
		return (
			this.currentRegistry !== undefined &&
			this.currentRegistry !== null &&
			this.currentRegistry !== this.defaultRegistry
		);
	}
}
