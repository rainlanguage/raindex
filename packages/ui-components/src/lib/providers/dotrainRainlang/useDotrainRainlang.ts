import { getDotrainRainlangContext, type DotrainRainlangContext } from './context';

/**
 * Hook to access the current Dotrain rainlang context.
 */
export function useDotrainRainlang() {
	return getDotrainRainlangContext();
}

if (import.meta.vitest) {
	const { describe, it, expect, vi, beforeEach } = import.meta.vitest;

	vi.mock('./context', () => ({
		getDotrainRainlangContext: vi.fn()
	}));

	describe('useDotrainRainlang', () => {
		const mockGetContext = vi.mocked(getDotrainRainlangContext);

		beforeEach(() => {
			mockGetContext.mockReset();
		});

		it('should return the rainlang context', () => {
			const mockContext: DotrainRainlangContext = {
				rainlang: null,
				manager: {
					getCurrentRainlang: vi.fn().mockReturnValue(''),
					setRainlang: vi.fn(),
					resetToDefault: vi.fn(),
					updateUrlWithRainlang: vi.fn(),
					isCustomRainlang: vi.fn().mockReturnValue(false)
				} as unknown as DotrainRainlangContext['manager']
			};
			mockGetContext.mockReturnValue(mockContext);

			const result = useDotrainRainlang();

			expect(mockGetContext).toHaveBeenCalled();
			expect(result).toEqual(mockContext);
		});
	});
}
