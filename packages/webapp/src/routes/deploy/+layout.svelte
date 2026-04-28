<script lang="ts">
	import CustomRegistryWarning from '$lib/components/CustomRegistryWarning.svelte';
	import {
		InputRegistryUrl,
		PageHeader,
		RegistryProvider,
		RegistryManager
	} from '@rainlanguage/ui-components';
	import { Toggle } from 'flowbite-svelte';
	import { page } from '$app/stores';
	import { REGISTRY_URL } from '$lib/constants';
	import { slide } from 'svelte/transition';
	let advancedMode = false;

	const registryManager = new RegistryManager(REGISTRY_URL);
	$: advancedMode = registryManager.isCustomRegistry();
	$: isDeployPage = $page.url.pathname === '/deploy';
</script>

<RegistryProvider {registryManager}>
	<PageHeader title={$page.data.pageName || 'Deploy'} pathname={$page.url.pathname} />
	<div class="flex flex-col gap-2">
		<div class="flex w-full content-end items-end justify-between">
			{#if advancedMode}
				<CustomRegistryWarning />
			{:else if isDeployPage}
				<div class="ml-auto"></div>
			{/if}
			{#if isDeployPage}
				<Toggle checked={advancedMode} on:change={() => (advancedMode = !advancedMode)}>
					<span class="whitespace-nowrap">Advanced mode</span>
				</Toggle>
			{/if}
		</div>
		<div class="flex flex-col items-end gap-4">
			{#if advancedMode && isDeployPage}
				<div in:slide class="w-full">
					<InputRegistryUrl />
				</div>
			{/if}
		</div>
	</div>
	<slot></slot>
</RegistryProvider>
