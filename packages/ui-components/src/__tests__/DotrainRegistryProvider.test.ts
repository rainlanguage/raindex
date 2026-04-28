import { render } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import DotrainRegistryProvider from '../lib/providers/dotrainRegistry/DotrainRegistryProvider.svelte';
import {
	setDotrainRegistryContext,
	type DotrainRegistryContext
} from '../lib/providers/dotrainRegistry/context';

vi.mock('../lib/providers/dotrainRegistry/context', () => ({
	setDotrainRegistryContext: vi.fn()
}));

describe('DotrainRegistryProvider', () => {
	const mockSetContext = vi.mocked(setDotrainRegistryContext);

	beforeEach(() => {
		mockSetContext.mockReset();
	});

	it('sets the registry context with provided props', () => {
		const registryUrl = 'https://example.com/registry.txt';
		const wasmErrorResult = {
			value: undefined,
			error: { msg: 'not implemented', readableMsg: 'not implemented' }
		};

		const registry: NonNullable<DotrainRegistryContext['registry']> = {
			free: vi.fn(),
			getAllOrderDetails: vi.fn(() => wasmErrorResult),
			getOrderKeys: vi.fn(() => ({ value: [], error: undefined })),
			getDeploymentDetails: vi.fn(() => wasmErrorResult),
			getGui: vi.fn(async () => wasmErrorResult),
			getRaindexYaml: vi.fn(() => wasmErrorResult),
			getRaindexClient: vi.fn(async () => wasmErrorResult),
			registryUrl,
			registry: 'registry-content',
			settingsUrl: 'https://example.com/settings.yaml',
			settings: 'settings-content',
			orderUrls: new Map<string, string>(),
			orders: new Map<string, string>()
		};
		const manager = {
			getCurrentRegistry: vi.fn().mockReturnValue(registryUrl),
			setRegistry: vi.fn(),
			resetToDefault: vi.fn(),
			updateUrlWithRegistry: vi.fn(),
			isCustomRegistry: vi.fn().mockReturnValue(false)
		} as unknown as DotrainRegistryContext['manager'];
		render(DotrainRegistryProvider, { props: { registry, manager, error: undefined } });

		expect(mockSetContext).toHaveBeenCalledWith({
			registry,
			manager,
			error: undefined
		});
	});
});
