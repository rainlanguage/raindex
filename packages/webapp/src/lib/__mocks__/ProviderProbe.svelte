<script lang="ts">
	export let registry: unknown = undefined;
	export let localDb: unknown = undefined;
	export let raindexClient: unknown = undefined;

	$: providerName =
		registry !== undefined
			? 'registry-provider'
			: localDb !== undefined
				? 'local-db-provider'
				: 'raindex-client-provider';
	$: providerValue = registry ?? localDb ?? raindexClient;
</script>

<div data-testid={providerName} data-ready={providerValue != null}>
	<slot />
</div>
