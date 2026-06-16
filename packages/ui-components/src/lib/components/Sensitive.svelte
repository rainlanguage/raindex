<script lang="ts">
	import { scrub as scrubStore } from '../storesGeneric/scrub';
	import type { Writable } from 'svelte/store';

	/**
	 * Wrap any content that may reveal an address or transaction hash. When the
	 * global `scrub` toggle is on, an opaque grey box is laid over the slotted
	 * content so it can't be read in a screenshot. Purely presentational - the
	 * underlying value is untouched.
	 *
	 * The store is injectable for testing; it defaults to the global scrub store.
	 */
	export let scrub: Writable<boolean> = scrubStore;
</script>

<span class="relative inline-block align-bottom" data-testid="sensitive">
	<span class:invisible={$scrub} data-testid="sensitive-content">
		<slot />
	</span>
	{#if $scrub}
		<span
			class="absolute inset-0 rounded bg-gray-400 dark:bg-gray-600"
			data-testid="sensitive-mask"
			aria-hidden="true"
		></span>
	{/if}
</span>
