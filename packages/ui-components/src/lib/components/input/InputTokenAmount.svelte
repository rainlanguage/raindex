<script lang="ts">
	import { InputAddon, Button } from 'flowbite-svelte';
	import { Float } from '@rainlanguage/raindex';

	export let symbol: string | undefined = undefined;
	export let maxValue: Float | undefined = undefined;
	export let value: Float = Float.parse('0').value as Float;

	let inputValue: string = '';

	function handleInput(event: Event) {
		const input = event.target as HTMLInputElement;
		inputValue = input.value;

		if (inputValue === '') {
			value = Float.parse('0').value as Float;
		} else {
			let parsedValue = Float.parse(inputValue);
			if (parsedValue.error) {
				value = Float.parse('0').value as Float;
				return;
			}
			value = parsedValue.value;
		}
	}

	function fillMaxValue() {
		if (!maxValue) return;
		value = maxValue;

		let formattedValue = value.format().value;
		if (!formattedValue) {
			inputValue = '';
			return;
		}

		inputValue = formattedValue as string;
	}
</script>

<div class="w-full">
	<div class="flex w-full">
		<div class="relative flex w-full">
			<input
				type="text"
				class={`focus:border-primary-500 focus:ring-primary-500 dark:focus:border-primary-500 dark:focus:ring-primary-500 block w-full rounded-lg border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 disabled:cursor-not-allowed disabled:opacity-50 rtl:text-right dark:border-gray-500 dark:bg-gray-600 dark:text-white dark:placeholder-gray-400 ${symbol && '!rounded-none !rounded-l-lg'}`}
				bind:value={inputValue}
				on:input={handleInput}
			/>

			{#if maxValue}
				<div class="absolute right-2 flex h-10 flex-col justify-center">
					<Button color="blue" class="px-2 py-1" size="xs" pill on:click={fillMaxValue}>MAX</Button>
				</div>
			{/if}
		</div>

		{#if symbol}
			<InputAddon>
				<span class="whitespace-nowrap">
					{symbol}
				</span>
			</InputAddon>
		{/if}
	</div>
</div>
