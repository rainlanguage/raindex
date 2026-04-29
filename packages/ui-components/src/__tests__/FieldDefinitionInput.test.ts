import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import FieldDefinitionInput from '../lib/components/deployment/FieldDefinitionInput.svelte';
import { DotrainOrderGui } from '@rainlanguage/raindex';
import userEvent from '@testing-library/user-event';
import { useGui } from '$lib/hooks/useGui';
import type { ComponentProps } from 'svelte';

type FieldDefinitionInputProps = ComponentProps<FieldDefinitionInput>;

vi.mock('@rainlanguage/raindex', () => ({
	DotrainOrderGui: vi.fn()
}));

vi.mock('$lib/hooks/useGui', () => ({
	useGui: vi.fn()
}));

describe('FieldDefinitionInput', () => {
	let guiInstance: DotrainOrderGui;
	let mockStateUpdateCallback: Mock;

	const mockFieldDefinition = {
		binding: 'test-binding',
		name: 'Test Field',
		description: 'Test Description',
		presets: [
			{ id: 'preset1', name: 'Preset 1', value: 'value1' },
			{ id: 'preset2', name: 'Preset 2', value: 'value2' }
		]
	};

	beforeEach(() => {
		mockStateUpdateCallback = vi.fn();

		guiInstance = {
			getFieldValue: vi.fn().mockReturnValue({}),
			setFieldValue: vi.fn().mockImplementation(() => {
				mockStateUpdateCallback();
			})
		} as unknown as DotrainOrderGui;

		(useGui as Mock).mockReturnValue(guiInstance);
	});

	it('renders field name and description', () => {
		const { getByText } = render(FieldDefinitionInput, {
			props: {
				fieldDefinition: mockFieldDefinition
			} as unknown as FieldDefinitionInputProps
		});

		expect(getByText('Test Field')).toBeTruthy();
		expect(getByText('Test Description')).toBeTruthy();
	});

	it('renders preset buttons', () => {
		const { getByText } = render(FieldDefinitionInput, {
			props: {
				fieldDefinition: mockFieldDefinition
			} as unknown as FieldDefinitionInputProps
		});

		expect(getByText('Preset 1')).toBeTruthy();
		expect(getByText('Preset 2')).toBeTruthy();
	});

	it('handles preset button clicks and triggers state update', async () => {
		const { getByText } = render(FieldDefinitionInput, {
			props: {
				fieldDefinition: mockFieldDefinition
			} as unknown as FieldDefinitionInputProps
		});

		await fireEvent.click(getByText('Preset 1'));

		expect(guiInstance.setFieldValue).toHaveBeenCalledWith('test-binding', 'value1');
		expect(mockStateUpdateCallback).toHaveBeenCalled();
	});

	it('handles custom input changes and triggers state update', async () => {
		const { getByPlaceholderText } = render(FieldDefinitionInput, {
			props: {
				fieldDefinition: { ...mockFieldDefinition, showCustomField: true }
			} as unknown as FieldDefinitionInputProps
		});

		const input = getByPlaceholderText('Enter custom value');
		await fireEvent.input(input, { target: { value: 'custom value' } });

		expect(guiInstance.setFieldValue).toHaveBeenCalledWith('test-binding', 'custom value');
		expect(mockStateUpdateCallback).toHaveBeenCalled();
	});

	it('does not show Custom button for is-fast-exit binding', () => {
		const fastExitFieldDef = {
			...mockFieldDefinition,
			binding: 'is-fast-exit'
		};

		const { queryByText } = render(FieldDefinitionInput, {
			props: {
				fieldDefinition: fastExitFieldDef
			} as unknown as FieldDefinitionInputProps
		});

		expect(queryByText('Custom')).toBeNull();
	});

	it('renders default value if it exists', async () => {
		const { getByPlaceholderText } = render(FieldDefinitionInput, {
			props: {
				fieldDefinition: {
					...mockFieldDefinition,
					default: 'default value',
					showCustomField: true
				}
			} as unknown as FieldDefinitionInputProps
		});

		const input = getByPlaceholderText('Enter custom value') as HTMLInputElement;
		expect(input.value).toBe('default value');

		await userEvent.type(input, '@');

		expect(guiInstance.setFieldValue).toHaveBeenCalledWith('test-binding', 'default value@');
	});
	it('renders selected value instead of default value', async () => {
		(guiInstance.getFieldValue as Mock).mockReturnValue({
			value: {
				binding: 'test-binding',
				value: 'preset1',
				is_preset: false
			}
		});

		const { getByPlaceholderText } = render(FieldDefinitionInput, {
			props: {
				fieldDefinition: {
					...mockFieldDefinition,
					default: 'default value',
					showCustomField: true
				}
			} as unknown as FieldDefinitionInputProps
		});

		const input = getByPlaceholderText('Enter custom value') as HTMLInputElement;
		expect(input.value).toBe('preset1');

		await userEvent.type(input, '@');

		expect(guiInstance.setFieldValue).toHaveBeenCalledWith('test-binding', 'preset1@');
	});
});
