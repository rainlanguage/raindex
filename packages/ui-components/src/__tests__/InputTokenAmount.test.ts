import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import InputTokenAmount from '$lib/components/input/InputTokenAmount.svelte';
import { Float } from '@rainlanguage/raindex';

vi.mock('@rainlanguage/raindex', async (importOriginal) => ({
	...(await importOriginal())
}));

describe('InputTokenAmount', () => {
	it('should handle empty input', async () => {
		const { getByRole, component } = render(InputTokenAmount, {
			props: { value: Float.parse('0').value }
		});
		const input = getByRole('textbox');

		await fireEvent.input(input, { target: { value: '' } });
		expect(component.$$.ctx[component.$$.props.value].format().value).toBe('0');
	});

	it('should handle invalid input', async () => {
		const { getByRole, component } = render(InputTokenAmount, {
			props: { value: Float.parse('0').value }
		});
		const input = getByRole('textbox');

		await fireEvent.input(input, { target: { value: 'abc' } });
		expect(component.$$.ctx[component.$$.props.value].format().value).toBe('0');
	});

	it('should handle maxValue prop', async () => {
		const { getByText, component } = render(InputTokenAmount, {
			props: { maxValue: Float.parse('1').value, value: Float.parse('0').value }
		});
		const maxButton = getByText('MAX');

		await fireEvent.click(maxButton);
		expect(component.$$.ctx[component.$$.props.value].format().value).toBe('1');
	});
});
