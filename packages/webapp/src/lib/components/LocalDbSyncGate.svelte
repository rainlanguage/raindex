<script lang="ts">
	import { localDbSyncGate } from '$lib/stores/localDbStatus';
</script>

{#if $localDbSyncGate.status === 'syncing'}
	<div
		data-testid="local-db-syncing-notice"
		class="mx-auto mt-12 flex max-w-2xl flex-col items-center rounded-lg border border-sky-200 bg-sky-50 px-6 py-8 text-center shadow-sm dark:border-sky-900/70 dark:bg-sky-950/30"
	>
		<div
			class="mb-5 h-10 w-10 animate-spin rounded-full border-2 border-sky-200 border-t-sky-600 dark:border-sky-900 dark:border-t-sky-400"
			aria-hidden="true"
		></div>
		<h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
			Local database syncing
		</h2>
		<p class="mt-2 max-w-lg text-sm leading-6 text-gray-600 dark:text-gray-300">
			We are preparing the local database. Orders and vaults will appear once the initial
			sync finishes.
		</p>
		{#if $localDbSyncGate.phaseMessage}
			<p class="mt-4 text-sm font-medium text-sky-700 dark:text-sky-300">
				{$localDbSyncGate.phaseMessage}
			</p>
		{/if}
	</div>
{:else if $localDbSyncGate.status === 'failure'}
	<div
		data-testid="local-db-sync-failure-notice"
		class="mx-auto mt-12 max-w-2xl rounded-lg border border-red-200 bg-red-50 px-6 py-8 text-center shadow-sm dark:border-red-900/70 dark:bg-red-950/30"
	>
		<h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
			Local database sync failed
		</h2>
		<p class="mt-2 text-sm leading-6 text-gray-600 dark:text-gray-300">
			Orders and vaults are unavailable until the local database finishes syncing.
		</p>
		{#if $localDbSyncGate.error}
			<p class="mt-4 text-sm font-medium text-red-700 dark:text-red-300">
				{$localDbSyncGate.error}
			</p>
		{/if}
	</div>
{:else}
	<slot />
{/if}
