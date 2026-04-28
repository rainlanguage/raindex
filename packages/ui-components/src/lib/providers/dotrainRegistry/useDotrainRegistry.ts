import { getDotrainRegistryContext, type DotrainRegistryContext } from './context';

/**
 * Hook to access the current Dotrain registry context.
 */
export function useDotrainRegistry() {
	return getDotrainRegistryContext();
}

if (import.meta.vitest) {
	const { describe, it, expect, vi, beforeEach } = import.meta.vitest;

	vi.mock('./context', () => ({
		getDotrainRegistryContext: vi.fn()
	}));

	describe('useDotrainRegistry', () => {
		const mockGetContext = vi.mocked(getDotrainRegistryContext);

		beforeEach(() => {
			mockGetContext.mockReset();
		});

		it('should return the registry context', () => {
			const mockContext: DotrainRegistryContext = {
				registry: null,
				manager: {
					getCurrentRegistry: vi.fn().mockReturnValue(''),
					setRegistry: vi.fn(),
					resetToDefault: vi.fn(),
					updateUrlWithRegistry: vi.fn(),
					isCustomRegistry: vi.fn().mockReturnValue(false)
				} as unknown as DotrainRegistryContext['manager']
			};
			mockGetContext.mockReturnValue(mockContext);

			const result = useDotrainRegistry();

			expect(mockGetContext).toHaveBeenCalled();
			expect(result).toEqual(mockContext);
		});
	});
}
