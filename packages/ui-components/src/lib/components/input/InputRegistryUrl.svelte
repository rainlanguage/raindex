<script lang="ts">
	import { Button, Input } from 'flowbite-svelte';
	import { useRegistry } from '$lib/providers/registry/useRegistry';
	import { loadRegistryUrl } from '$lib/services/loadRegistryUrl';

	const registry = useRegistry();
	let newRegistryUrl = registry.getCurrentRegistry();
	let error: string | null = null;
	let loading: boolean = false;

	async function handleClick() {
		loading = true;
		error = null;
		try {
			if (!registry) {
				throw new Error('Registry manager not yet available.');
			}
			await loadRegistryUrl(newRegistryUrl, registry);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Unknown error';
		}
		loading = false;
	}
</script>

<div class="flex w-full flex-col items-end gap-2">
	<div class="flex w-full items-start gap-4" data-testid="registry-input">
		<Input
			id="order-url"
			type="url"
			placeholder="Enter URL to raw order registry file"
			bind:value={newRegistryUrl}
		/>
		<Button class="w-36 text-nowrap" on:click={handleClick} disabled={loading}>
			{loading ? 'Loading registry...' : 'Load registry URL'}
		</Button>
	</div>
	<div class="h-4">
		{#if error}
			<p data-testid="registry-error" class="text-red-500">{error}</p>
		{/if}
	</div>
</div>
