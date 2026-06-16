<script lang="ts">
	import { Button, Input } from 'flowbite-svelte';
	import { useRainlang } from '$lib/providers/rainlang/useRainlang';
	import { loadRainlangUrl } from '$lib/services/loadRainlangUrl';

	const rainlang = useRainlang();
	let newRainlangUrl = rainlang.getCurrentRainlang();
	let error: string | null = null;
	let loading: boolean = false;

	async function handleClick() {
		loading = true;
		error = null;
		try {
			if (!rainlang) {
				throw new Error('Rainlang manager not yet available.');
			}
			await loadRainlangUrl(newRainlangUrl, rainlang);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Unknown error';
		}
		loading = false;
	}
</script>

<div class="flex w-full flex-col items-end gap-2">
	<div class="flex w-full items-start gap-4" data-testid="rainlang-input">
		<Input
			id="order-url"
			type="url"
			placeholder="Enter URL to raw order rainlang file"
			bind:value={newRainlangUrl}
		/>
		<Button class="w-36 text-nowrap" on:click={handleClick} disabled={loading}>
			{loading ? 'Loading rainlang...' : 'Load rainlang URL'}
		</Button>
	</div>
	<div class="h-4">
		{#if error}
			<p data-testid="rainlang-error" class="text-red-500">{error}</p>
		{/if}
	</div>
</div>
