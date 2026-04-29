import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import ComposedRainlangModal from '../lib/components/deployment/ComposedRainlangModal.svelte';
import type { DotrainOrderGui } from '@rainlanguage/raindex';
import { useGui } from '$lib/hooks/useGui';

vi.mock('svelte-codemirror-editor', async () => {
	const mockCodeMirror = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { default: mockCodeMirror };
});

vi.mock('$lib/hooks/useGui', () => ({
	useGui: vi.fn()
}));

const mockGui = {
	getComposedRainlang: vi.fn(() => Promise.resolve('mocked rainlang text'))
} as unknown as DotrainOrderGui;

describe('ComposedRainlangModal', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		(useGui as Mock).mockReturnValue(mockGui);
	});

	it('should open modal and display rainlang text when button is clicked', async () => {
		const { getByText, getByTestId } = render(ComposedRainlangModal);

		const button = getByText('Show Rainlang');
		await fireEvent.click(button);

		await waitFor(() => {
			expect(mockGui.getComposedRainlang).toHaveBeenCalled();
			expect(getByTestId('modal')).toBeInTheDocument();
		});
	});
});
