import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import ComposedRainlangModal from '../lib/components/deployment/ComposedRainlangModal.svelte';
import type { RaindexOrderBuilder } from '@rainlanguage/orderbook';
import { useRaindexOrderBuilder } from '$lib/hooks/useRaindexOrderBuilder';

vi.mock('svelte-codemirror-editor', async () => {
	const mockCodeMirror = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { default: mockCodeMirror };
});

vi.mock('$lib/hooks/useRaindexOrderBuilder', () => ({
	useRaindexOrderBuilder: vi.fn()
}));

const mockBuilder = {
	getComposedRainlang: vi.fn(() => Promise.resolve('mocked rainlang text'))
} as unknown as RaindexOrderBuilder;

describe('ComposedRainlangModal', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		(useRaindexOrderBuilder as Mock).mockReturnValue(mockBuilder);
	});

	it('should open modal and display rainlang text when button is clicked', async () => {
		const { getByText, getByTestId } = render(ComposedRainlangModal);

		const button = getByText('Show Rainlang');
		await fireEvent.click(button);

		await waitFor(() => {
			expect(mockBuilder.getComposedRainlang).toHaveBeenCalled();
			expect(getByTestId('modal')).toBeInTheDocument();
		});
	});
});
