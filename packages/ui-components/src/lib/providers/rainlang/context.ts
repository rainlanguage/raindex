import { getContext, setContext } from 'svelte';
import type { RainlangManager } from './RainlangManager';

export const RAINLANG_KEY = 'rainlang_key';
/**
 * Retrieves the rainlang manager directly from Svelte's context
 */
export const getRainlangContext = (): RainlangManager => {
	const rainlang = getContext<RainlangManager>(RAINLANG_KEY);
	if (!rainlang) {
		throw new Error(
			'No rainlang manager was found in Svelte context. Did you forget to wrap your component with RainlangProvider?'
		);
	}
	return rainlang;
};

/**
 * Sets the rainlang manager in Svelte's context
 */
export const setRainlangContext = (rainlang: RainlangManager) => {
	setContext(RAINLANG_KEY, rainlang);
};

if (import.meta.vitest) {
	const { describe, it, expect, vi, beforeEach } = import.meta.vitest;

	vi.mock('svelte', async (importOriginal) => ({
		...((await importOriginal()) as object),
		getContext: vi.fn()
	}));

	describe('getRainlangContext', () => {
		const mockGetContext = vi.mocked(getContext);

		beforeEach(() => {
			mockGetContext.mockReset();
		});

		it('should return the rainlang from context when it exists', () => {
			const mockRainlang = {} as RainlangManager;

			mockGetContext.mockImplementation((key) => {
				if (key === RAINLANG_KEY) return mockRainlang;
				return undefined;
			});

			const result = getRainlangContext();
			expect(mockGetContext).toHaveBeenCalledWith(RAINLANG_KEY);
			expect(result).toEqual(mockRainlang);
		});

		it('should throw an error when rainlang is not in context', () => {
			mockGetContext.mockReturnValue(undefined);

			expect(() => getRainlangContext()).toThrow(
				'No rainlang manager was found in Svelte context. Did you forget to wrap your component with RainlangProvider?'
			);
		});
	});
}
