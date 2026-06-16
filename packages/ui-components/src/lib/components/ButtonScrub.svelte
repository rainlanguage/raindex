<script lang="ts">
	import { EyeOutline, EyeSlashOutline } from 'flowbite-svelte-icons';
	import { scrub as scrubStore } from '../storesGeneric/scrub';
	import type { Writable } from 'svelte/store';

	/**
	 * Eyeball toggle for the global "scrub" mode. Sits next to the dark-mode
	 * button in the sidebar. Clicking it masks/unmasks every address and
	 * transaction hash in the GUI for safe screenshotting.
	 *
	 * The store is injectable for testing; it defaults to the global scrub store.
	 */
	export let scrub: Writable<boolean> = scrubStore;

	function toggle() {
		scrub.update((value) => !value);
	}
</script>

<button
	type="button"
	on:click={toggle}
	data-testid="scrub-toggle"
	aria-pressed={$scrub}
	title={$scrub ? 'Show addresses and hashes' : 'Hide addresses and hashes for screenshots'}
	class="rounded-lg p-2 text-gray-500 hover:bg-gray-100 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-white"
>
	{#if $scrub}
		<EyeSlashOutline />
	{:else}
		<EyeOutline />
	{/if}
</button>
