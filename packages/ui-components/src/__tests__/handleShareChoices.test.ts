import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { handleShareChoices } from '../lib/services/handleShareChoices';
import { DotrainOrderGui } from '@rainlanguage/raindex';

vi.mock('@rainlanguage/raindex', () => ({
	DotrainOrderGui: vi.fn()
}));

describe('handleShareChoices', () => {
	let guiInstance: DotrainOrderGui;
	const mockRegistryUrl = 'https://example.com/registry';

	beforeEach(() => {
		guiInstance = {
			serializeState: vi.fn()
		} as unknown as DotrainOrderGui;

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
		(guiInstance.serializeState as Mock).mockReturnValue({ value: 'mockState123' });

		await handleShareChoices(guiInstance, mockRegistryUrl);

		expect(navigator.clipboard.writeText).toHaveBeenCalledWith(
			'http://example.com/?state=mockState123&registry=https%3A%2F%2Fexample.com%2Fregistry'
		);
	});

	it('should handle null state', async () => {
		(guiInstance.serializeState as Mock).mockReturnValue({ value: null });

		await handleShareChoices(guiInstance, mockRegistryUrl);

		expect(navigator.clipboard.writeText).toHaveBeenCalledWith(
			'http://example.com/?state=&registry=https%3A%2F%2Fexample.com%2Fregistry'
		);
	});
});
