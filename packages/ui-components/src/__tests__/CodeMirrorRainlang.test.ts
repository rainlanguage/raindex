import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, waitFor } from '@testing-library/svelte';
import CodeMirrorRainlang from '../lib/components/CodeMirrorRainlang.svelte';
import type { RaindexOrder } from '@rainlanguage/raindex';
import { writable } from 'svelte/store';

vi.mock('codemirror-rainlang', () => ({
	RainlangLR: vi.fn()
}));

vi.mock('svelte-codemirror-editor', async () => {
	const mockCodeMirror = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { default: mockCodeMirror };
});

describe('CodeMirrorRainlang', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should use order.rainlang when order prop is provided', async () => {
		const mockOrder: RaindexOrder = {
			rainlang: 'mocked rainlang text'
		} as RaindexOrder;

		const screen = render(CodeMirrorRainlang, {
			props: {
				order: mockOrder,
				codeMirrorTheme: writable({}),
				codeMirrorDisabled: false,
				codeMirrorStyles: {}
			}
		});

		await waitFor(() => {
			const mockComponent = screen.getByTestId('mock-component');
			expect(mockComponent).toHaveAttribute('value', 'mocked rainlang text');
		});
	});

	it('should use rainlangText when no order is provided', async () => {
		const testText = 'test rainlang text';

		const screen = render(CodeMirrorRainlang, {
			props: {
				order: undefined,
				rainlangText: testText,
				codeMirrorTheme: writable({}),
				codeMirrorDisabled: false,
				codeMirrorStyles: {}
			}
		});

		await waitFor(() => {
			const mockComponent = screen.getByTestId('mock-component');
			expect(mockComponent).toHaveAttribute('value', 'test rainlang text');
		});
	});

	it('should pass through disabled prop', async () => {
		const mockOrder: RaindexOrder = {
			rainlang: 'mocked rainlang text'
		} as RaindexOrder;

		const screen = render(CodeMirrorRainlang, {
			props: {
				order: mockOrder,
				codeMirrorTheme: writable({}),
				codeMirrorDisabled: false,
				codeMirrorStyles: {}
			}
		});

		await waitFor(() => {
			const mockComponent = screen.getByTestId('mock-component');
			expect(mockComponent).toHaveAttribute('readonly', 'false');
		});
	});
});
