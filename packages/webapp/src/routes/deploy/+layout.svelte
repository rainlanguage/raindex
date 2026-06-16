<script lang="ts">
	import CustomRainlangWarning from '$lib/components/CustomRainlangWarning.svelte';
	import {
		InputRainlangUrl,
		PageHeader,
		RainlangProvider,
		RainlangManager
	} from '@rainlanguage/ui-components';
	import { Toggle } from 'flowbite-svelte';
	import { page } from '$app/stores';
	import { RAINLANG_URL } from '$lib/constants';
	import { slide } from 'svelte/transition';
	let advancedMode = false;

	const rainlangManager = new RainlangManager(RAINLANG_URL);
	$: advancedMode = rainlangManager.isCustomRainlang();
	$: isDeployPage = $page.url.pathname === '/deploy';
</script>

<RainlangProvider {rainlangManager}>
	<PageHeader title={$page.data.pageName || 'Deploy'} pathname={$page.url.pathname} />
	<div class="flex flex-col gap-2">
		<div class="flex w-full content-end items-end justify-between">
			{#if advancedMode}
				<CustomRainlangWarning />
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
					<InputRainlangUrl />
				</div>
			{/if}
		</div>
	</div>
	<slot></slot>
</RainlangProvider>
