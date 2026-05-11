import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { handleShareChoices } from '../lib/services/handleShareChoices';
import { RaindexOrderBuilder } from '@rainlanguage/orderbook';

vi.mock('@rainlanguage/orderbook', () => ({
	RaindexOrderBuilder: vi.fn()
}));

describe('handleShareChoices', () => {
	let builderInstance: RaindexOrderBuilder;
	const mockRegistryUrl = 'https://example.com/registry';

	beforeEach(() => {
		builderInstance = {
			serializeState: vi.fn()
		} as unknown as RaindexOrderBuilder;

		Object.assign(navigator, {
			clipboard: {
				writeText: vi.fn()
			}
		});
		vi.mock('$app/stores', () => ({
			page: {
				subscribe: vi.fn((fn) => {
					fn({ url: new URL('http://example.com') });
					return () => {};
				})
			}
		}));
	});

	it('should share the choices with state and registry', async () => {
		(builderInstance.serializeState as Mock).mockReturnValue({ value: 'mockState123' });

		await handleShareChoices(builderInstance, mockRegistryUrl);

		expect(navigator.clipboard.writeText).toHaveBeenCalledWith(
			'http://example.com/?state=mockState123&registry=https%3A%2F%2Fexample.com%2Fregistry'
		);
	});

	it('should handle null state', async () => {
		(builderInstance.serializeState as Mock).mockReturnValue({ value: null });

		await handleShareChoices(builderInstance, mockRegistryUrl);

		expect(navigator.clipboard.writeText).toHaveBeenCalledWith(
			'http://example.com/?state=&registry=https%3A%2F%2Fexample.com%2Fregistry'
		);
	});
});
