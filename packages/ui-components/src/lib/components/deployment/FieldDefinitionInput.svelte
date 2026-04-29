<script lang="ts">
	import { Input } from 'flowbite-svelte';
	import {
		type FieldValue,
		type OrderBuilderFieldDefinitionCfg,
		type OrderBuilderPresetCfg
	} from '@rainlanguage/orderbook';
	import ButtonSelectOption from './ButtonSelectOption.svelte';
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';
	import { onMount } from 'svelte';
	import { useRaindexOrderBuilder } from '$lib/hooks/useRaindexOrderBuilder';

	export let fieldDefinition: OrderBuilderFieldDefinitionCfg;

	const builder = useRaindexOrderBuilder();

	let currentValue: FieldValue | undefined;
	let inputValue: string | null = currentValue?.value
		? currentValue?.value
		: fieldDefinition.default || null;

	onMount(() => {
		try {
			const result = builder.getFieldValue(fieldDefinition.binding);
			if (result.error) {
				throw new Error(result.error.msg);
			}
			currentValue = result.value;
			inputValue = currentValue?.value ? currentValue?.value : fieldDefinition.default || null;
		} catch {
			currentValue = undefined;
		}
	});

	async function handlePresetClick(preset: OrderBuilderPresetCfg) {
		inputValue = preset.value;
		builder.setFieldValue(fieldDefinition.binding, inputValue);

		const result = builder.getFieldValue(fieldDefinition.binding);
		if (result.error) {
			throw new Error(result.error.msg);
		}
		currentValue = result.value;
	}

	async function handleCustomInputChange(value: string) {
		inputValue = value;
		builder.setFieldValue(fieldDefinition.binding, inputValue);

		const result = builder.getFieldValue(fieldDefinition.binding);
		if (result.error) {
			throw new Error(result.error.msg);
		}
		currentValue = result.value;
	}
</script>

<div class="flex w-full flex-col gap-6">
	<DeploymentSectionHeader title={fieldDefinition.name} description={fieldDefinition.description} />
	<div class="flex w-full flex-col gap-6">
		{#if fieldDefinition.presets}
			<div class="flex w-full flex-wrap gap-4">
				{#each fieldDefinition.presets as preset}
					<ButtonSelectOption
						dataTestId={`binding-${fieldDefinition.binding}-preset-${preset.name || preset.value}`}
						buttonText={preset.name || preset.value}
						clickHandler={() => handlePresetClick(preset)}
						active={currentValue?.value === preset.value}
					/>
				{/each}
			</div>
		{/if}

		{#if !fieldDefinition.presets || fieldDefinition.showCustomField}
			<Input
				data-testid={`binding-${fieldDefinition.binding}-input`}
				size="lg"
				placeholder="Enter custom value"
				bind:value={inputValue}
				on:input={({ currentTarget }) => {
					if (currentTarget instanceof HTMLInputElement) {
						handleCustomInputChange(currentTarget.value);
					}
				}}
			/>
		{/if}
	</div>
</div>
