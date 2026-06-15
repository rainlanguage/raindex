<script lang="ts">
	import { DarkMode } from 'flowbite-svelte';
	import { writable } from 'svelte/store';
	export let colorTheme = writable();
	function toggle(e: MouseEvent) {
		const isDark = (e.target as HTMLElement).ownerDocument.documentElement.classList.contains(
			'dark'
		);
		colorTheme.set(isDark ? 'dark' : 'light');
	}
</script>

<button
	type="button"
	title="Toggle dark mode"
	aria-label="Toggle dark mode"
	data-testid="dark-mode-toggle"
	on:click={toggle}
>
	<DarkMode />
</button>
