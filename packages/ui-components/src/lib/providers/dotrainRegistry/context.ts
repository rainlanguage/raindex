import { getContext, setContext } from 'svelte';
import type { DotrainRegistry } from '@rainlanguage/raindex';
import type { RegistryManager } from '../registry/RegistryManager';

export type DotrainRegistryContext = {
	registry: DotrainRegistry | null;
	error?: string;
	manager: RegistryManager;
};

const DOTRAIN_REGISTRY_CONTEXT_KEY = 'dotrain-registry-context';

export const setDotrainRegistryContext = (context: DotrainRegistryContext) => {
	setContext(DOTRAIN_REGISTRY_CONTEXT_KEY, context);
};

export const getDotrainRegistryContext = (): DotrainRegistryContext => {
	const ctx = getContext<DotrainRegistryContext>(DOTRAIN_REGISTRY_CONTEXT_KEY);
	if (!ctx) {
		throw new Error(
			'Dotrain registry context not found. Did you forget to wrap your app in DotrainRegistryProvider?'
		);
	}
	return ctx;
};

if (import.meta.vitest) {
	const { describe, it, expect, vi, beforeEach } = import.meta.vitest;

	vi.mock('svelte', async (importOriginal) => ({
		...((await importOriginal()) as object),
		getContext: vi.fn()
	}));

	describe('getDotrainRegistryContext', () => {
		const mockGetContext = vi.mocked(getContext);

		beforeEach(() => {
			mockGetContext.mockReset();
		});

		it('should return the context when it exists', () => {
			const mockCtx = { registry: null, manager: {} as RegistryManager } as DotrainRegistryContext;

			mockGetContext.mockImplementation((key) => {
				if (key === DOTRAIN_REGISTRY_CONTEXT_KEY) return mockCtx;
				return undefined;
			});

			const result = getDotrainRegistryContext();
			expect(mockGetContext).toHaveBeenCalledWith(DOTRAIN_REGISTRY_CONTEXT_KEY);
			expect(result).toEqual(mockCtx);
		});

		it('should throw an error when context is not set', () => {
			mockGetContext.mockReturnValue(undefined);

			expect(() => getDotrainRegistryContext()).toThrow(
				'Dotrain registry context not found. Did you forget to wrap your app in DotrainRegistryProvider?'
			);
		});
	});
}
