<script lang="ts">
	import type { NameAndDescriptionCfg } from '@rainlanguage/raindex';
	import DeploymentTile from './DeploymentTile.svelte';

	export let deployments: Map<string, NameAndDescriptionCfg> | [string, NameAndDescriptionCfg][] =
		[];
	export let orderName: string;

	$: deploymentEntries =
		deployments instanceof Map ? Array.from(deployments.entries()) : (deployments ?? []);
</script>

{#if deploymentEntries.length === 0}
	<p class="text-gray-500">No deployments found for this order.</p>
{:else}
	<div
		class="mr-auto grid w-full grid-cols-1 justify-items-start gap-4 md:grid-cols-2 lg:w-auto lg:grid-cols-3"
	>
		{#each deploymentEntries as [key, { name, description }]}
			<DeploymentTile {name} {description} {key} {orderName} />
		{/each}
	</div>
{/if}
