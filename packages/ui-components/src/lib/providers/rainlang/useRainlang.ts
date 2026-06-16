import { getRainlangContext } from './context';
import type { RainlangManager } from './RainlangManager';

/**
 * Hook to access rainlang manager information from context
 * Must be used within a component that is a child of RainlangProvider
 * @returns An object containing the rainlang manager
 */
export function useRainlang() {
	const rainlang = getRainlangContext();
	return rainlang;
}

if (import.meta.vitest) {
	const { describe, it, expect, vi, beforeEach } = import.meta.vitest;

	vi.mock('./context', () => ({
		getRainlangContext: vi.fn()
	}));

	describe('useRainlang', () => {
		const mockGetRainlangContext = vi.mocked(getRainlangContext);

		beforeEach(() => {
			mockGetRainlangContext.mockReset();
		});

		it('should return rainlang', () => {
			const mockRainlang = {} as RainlangManager;
			mockGetRainlangContext.mockReturnValue(mockRainlang);

			const result = useRainlang();

			expect(mockGetRainlangContext).toHaveBeenCalled();
			expect(result).toEqual(mockRainlang);
		});
	});
}
