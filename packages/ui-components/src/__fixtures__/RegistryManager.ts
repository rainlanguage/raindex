import { vi } from 'vitest';
import type { RegistryManager } from '$lib/providers/registry/RegistryManager';

const mockDefaultRegistry = 'https://example.com/default-registry.json';
let mockCurrentRegistry: string | null = mockDefaultRegistry;

export const initialRegistry: Partial<RegistryManager> = {
	getCurrentRegistry: vi.fn(() => mockCurrentRegistry ?? mockDefaultRegistry),
	setRegistry: vi.fn((newRegistry: string) => {
		mockCurrentRegistry = newRegistry;
	}),
	resetToDefault: vi.fn(() => {
		mockCurrentRegistry = mockDefaultRegistry;
	}),
	updateUrlWithRegistry: vi.fn(),
	isCustomRegistry: vi.fn(() => mockCurrentRegistry !== mockDefaultRegistry)
};
