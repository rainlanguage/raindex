import { vi } from 'vitest';
import type { RainlangManager } from '$lib/providers/rainlang/RainlangManager';

const mockDefaultRainlang = 'https://example.com/default-rainlang.json';
let mockCurrentRainlang: string | null = mockDefaultRainlang;

export const initialRainlang: Partial<RainlangManager> = {
	getCurrentRainlang: vi.fn(() => mockCurrentRainlang ?? mockDefaultRainlang),
	setRainlang: vi.fn((newRainlang: string) => {
		mockCurrentRainlang = newRainlang;
	}),
	resetToDefault: vi.fn(() => {
		mockCurrentRainlang = mockDefaultRainlang;
	}),
	updateUrlWithRainlang: vi.fn(),
	isCustomRainlang: vi.fn(() => mockCurrentRainlang !== mockDefaultRainlang)
};
