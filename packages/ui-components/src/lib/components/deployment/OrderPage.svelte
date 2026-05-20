<script lang="ts">
	import { fade } from 'svelte/transition';
	import type { NameAndDescriptionCfg } from '@rainlanguage/raindex';
	import DeploymentsSection from './DeploymentsSection.svelte';
	import SvelteMarkdown from 'svelte-markdown';
	import { onMount } from 'svelte';

	export let orderName: string = '';
	export let orderDetail: NameAndDescriptionCfg;
	export let deployments: Map<string, NameAndDescriptionCfg> | [string, NameAndDescriptionCfg][] =
		[];

	let markdownContent: string = '';
	let error: string | undefined;

	const isMarkdownUrl = (url: string): boolean => {
		return url.trim().toLowerCase().endsWith('.md');
	};

	const fetchMarkdownContent = async (url: string) => {
		try {
			const response = await fetch(url);
			if (response.ok) {
				markdownContent = await response.text();
			}
		} catch {
			error = `Failed to fetch markdown`;
		}
	};

	onMount(async () => {
		if (orderDetail?.description && isMarkdownUrl(orderDetail.description)) {
			await fetchMarkdownContent(orderDetail.description);
		}
	});
</script>

{#if orderDetail}
	<div>
		<div in:fade class="flex flex-col gap-8">
			<div class="flex max-w-2xl flex-col gap-3 text-start lg:gap-6">
				<h1 class="text-4xl font-semibold text-gray-900 lg:text-6xl dark:text-white">
					{orderDetail.name ?? orderName}
				</h1>
				{#if markdownContent}
					<div data-testid="markdown-content" class="prose dark:prose-invert">
						<SvelteMarkdown source={markdownContent} />
					</div>
				{:else}
					<div class="flex flex-col gap-2">
						{#if error}
							<p data-testid="markdown-error" class="text-red-500">{error}</p>
						{/if}
						<p
							data-testid="plain-description"
							class="text-base text-gray-600 lg:text-lg dark:text-gray-400"
						>
							{orderDetail.description}
						</p>
					</div>
				{/if}
			</div>
			<div class="u flex flex-col gap-4">
				<h2 class="text-3xl font-semibold text-gray-900 dark:text-white">Deployments</h2>
				<DeploymentsSection {deployments} {orderName} />
			</div>
		</div>
	</div>
{:else}
	<div>
		<p class="text-red-500">Failed to load order details.</p>
	</div>
{/if}
