<script lang="ts">
	import { getContext } from 'svelte';
	import type { AriaRole } from 'svelte/elements';
	import { twMerge } from 'tailwind-merge';

	const ctx: { size: 'xs' | 'sm' | 'md' | 'lg' | 'xl'; role: AriaRole | undefined } =
		getContext('iconCtx') ?? {};
	const sizes = {
		xs: 'w-3 h-3',
		sm: 'w-4 h-4',
		md: 'w-5 h-5',
		lg: 'w-6 h-6',
		xl: 'w-8 h-8'
	};
	export let size = ctx.size || 'md';
	export let role = ctx.role || 'img';
	export let ariaLabel = 'refresh';
	export let title = 'Refresh';
	export let spin = false;
	export let testId = 'refresh-button';
</script>

<svg
	data-testid={testId}
	xmlns="http://www.w3.org/2000/svg"
	fill="none"
	{...{ title, ...$$restProps }}
	class={twMerge(
		'shrink-0 cursor-pointer outline-none',
		sizes[size],
		$$props.class,
		spin ? 'animate-spin ease-out' : ''
	)}
	{role}
	aria-label={ariaLabel}
	viewBox="0 0 24 24"
	on:click
	on:keydown
	on:keyup
	on:focus
	on:blur
	on:mouseenter
	on:mouseleave
	on:mouseover
	on:mouseout
>
	<path
		stroke="currentColor"
		stroke-linecap="round"
		stroke-linejoin="round"
		stroke-width="2"
		d="M17.651 7.65a7.131 7.131 0 0 0-12.68 3.15M18.001 4v4h-4m-7.652 8.35a7.13 7.13 0 0 0 12.68-3.15M6 20v-4h4"
	/>
</svg>

<!--
    @component
    [Go to docs](https://flowbite-svelte-icons.vercel.app/)
    ## Props
    @prop export let size: 'xs' | 'sm' | 'md' | 'lg' | 'xl' = ctx.size || 'md';
    @prop export let role = ctx.role || 'img';
    @prop export let ariaLabel = 'refresh';
    -->
