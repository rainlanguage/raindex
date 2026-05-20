<script lang="ts">
	import type { LocalDbStatus } from '@rainlanguage/raindex';

	const STATUS_LABELS: Record<LocalDbStatus, string> = {
		active: 'Active',
		syncing: 'Syncing',
		failure: 'Failure'
	};

	const DOT_CLASSES: Record<LocalDbStatus, string> = {
		active: 'bg-emerald-400',
		syncing: 'bg-sky-400',
		failure: 'bg-red-500'
	};

	const TEXT_CLASSES: Record<LocalDbStatus, string> = {
		active: 'text-emerald-400',
		syncing: 'text-sky-400',
		failure: 'text-red-400'
	};

	export let status: LocalDbStatus = 'active';
</script>

<span
	class="inline-flex items-center gap-2 rounded-full border border-gray-200 bg-gray-100 px-3 py-1 text-xs font-semibold uppercase tracking-wide text-gray-600 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-300"
	data-testid="local-db-status"
>
	<span class={`h-2 w-2 rounded-full ${DOT_CLASSES[status]}`} aria-hidden="true" />
	<span class={TEXT_CLASSES[status]}>
		{STATUS_LABELS[status]}
	</span>
</span>
