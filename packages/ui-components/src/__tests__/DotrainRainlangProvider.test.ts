import { render } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import DotrainRainlangProvider from '../lib/providers/dotrainRainlang/DotrainRainlangProvider.svelte';
import {
	setDotrainRainlangContext,
	type DotrainRainlangContext
} from '../lib/providers/dotrainRainlang/context';

vi.mock('../lib/providers/dotrainRainlang/context', () => ({
	setDotrainRainlangContext: vi.fn()
}));

describe('DotrainRainlangProvider', () => {
	const mockSetContext = vi.mocked(setDotrainRainlangContext);

	beforeEach(() => {
		mockSetContext.mockReset();
	});

	it('sets the rainlang context with provided props', () => {
		const rainlangUrl = 'https://example.com/rainlang.txt';
		const wasmErrorResult = {
			value: undefined,
			error: { msg: 'not implemented', readableMsg: 'not implemented' }
		};

		const rainlang: NonNullable<DotrainRainlangContext['rainlang']> = {
			free: vi.fn(),
			getAllOrderDetails: vi.fn(() => wasmErrorResult),
			getOrderKeys: vi.fn(() => ({ value: [], error: undefined })),
			getDeploymentDetails: vi.fn(() => wasmErrorResult),
			getGui: vi.fn(async () => wasmErrorResult),
			getOrderbookYaml: vi.fn(() => wasmErrorResult),
			getRaindexClient: vi.fn(async () => wasmErrorResult),
			rainlangUrl,
			rainlang: 'rainlang-content',
			settingsUrl: 'https://example.com/settings.yaml',
			settings: 'settings-content',
			orderUrls: new Map<string, string>(),
			orders: new Map<string, string>()
		};
		const manager = {
			getCurrentRainlang: vi.fn().mockReturnValue(rainlangUrl),
			setRainlang: vi.fn(),
			resetToDefault: vi.fn(),
			updateUrlWithRainlang: vi.fn(),
			isCustomRainlang: vi.fn().mockReturnValue(false)
		} as unknown as DotrainRainlangContext['manager'];
		render(DotrainRainlangProvider, { props: { rainlang, manager, error: undefined } });

		expect(mockSetContext).toHaveBeenCalledWith({
			rainlang,
			manager,
			error: undefined
		});
	});
});
