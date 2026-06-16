import { getContext, setContext } from 'svelte';
import type { DotrainRainlang } from '@rainlanguage/orderbook';
import type { RainlangManager } from '../rainlang/RainlangManager';

export type DotrainRainlangContext = {
	rainlang: DotrainRainlang | null;
	error?: string;
	manager: RainlangManager;
};

const DOTRAIN_RAINLANG_CONTEXT_KEY = 'dotrain-rainlang-context';

export const setDotrainRainlangContext = (context: DotrainRainlangContext) => {
	setContext(DOTRAIN_RAINLANG_CONTEXT_KEY, context);
};

export const getDotrainRainlangContext = (): DotrainRainlangContext => {
	const ctx = getContext<DotrainRainlangContext>(DOTRAIN_RAINLANG_CONTEXT_KEY);
	if (!ctx) {
		throw new Error(
			'Dotrain rainlang context not found. Did you forget to wrap your app in DotrainRainlangProvider?'
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

	describe('getDotrainRainlangContext', () => {
		const mockGetContext = vi.mocked(getContext);

		beforeEach(() => {
			mockGetContext.mockReset();
		});

		it('should return the context when it exists', () => {
			const mockCtx = { rainlang: null, manager: {} as RainlangManager } as DotrainRainlangContext;

			mockGetContext.mockImplementation((key) => {
				if (key === DOTRAIN_RAINLANG_CONTEXT_KEY) return mockCtx;
				return undefined;
			});

			const result = getDotrainRainlangContext();
			expect(mockGetContext).toHaveBeenCalledWith(DOTRAIN_RAINLANG_CONTEXT_KEY);
			expect(result).toEqual(mockCtx);
		});

		it('should throw an error when context is not set', () => {
			mockGetContext.mockReturnValue(undefined);

			expect(() => getDotrainRainlangContext()).toThrow(
				'Dotrain rainlang context not found. Did you forget to wrap your app in DotrainRainlangProvider?'
			);
		});
	});
}
