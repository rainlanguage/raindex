<script lang="ts">
	import { useRegistry } from '@rainlanguage/ui-components';
	const registry = useRegistry();
</script>

<div
	class="flex items-center gap-x-2 rounded-lg bg-yellow-50 p-4 text-sm text-yellow-800 dark:bg-yellow-800/30 dark:text-yellow-300"
	data-testid="custom-registry-warning"
>
	<svg
		class="h-5 w-5 flex-shrink-0"
		fill="currentColor"
		viewBox="0 0 20 20"
		xmlns="http://www.w3.org/2000/svg"
	>
		<path
			fill-rule="evenodd"
			d="M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.17 2.625-1.516 2.625H3.72c-1.347 0-2.189-1.458-1.515-2.625L8.485 2.495zM10 5a.75.75 0 01.75.75v3.5a.75.75 0 01-1.5 0v-3.5A.75.75 0 0110 5zm0 9a1 1 0 100-2 1 1 0 000 2z"
			clip-rule="evenodd"
		/>
	</svg>
	<span>
		You are using a <span class="font-medium">custom orders registry. </span>
		<a
			on:click={() => {
				registry.resetToDefault();
			}}
			href="/deploy"
			data-sveltekit-reload
			class="font-semibold underline hover:text-blue-800 dark:hover:text-blue-900"
		>
			Use default.
		</a>
	</span>
</div>
