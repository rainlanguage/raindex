<script lang="ts">
	export let registry: unknown = undefined;
	export let localDb: unknown = undefined;
	export let raindexClient: unknown = undefined;
	export let error: unknown = undefined;
	export let manager: unknown = undefined;

	const providerName =
		registry !== undefined
			? 'registry-provider'
			: localDb !== undefined
				? 'local-db-provider'
				: 'raindex-client-provider';
	const providerValue = registry ?? localDb ?? raindexClient;
	const resourceId = (providerValue as { id?: string } | null)?.id ?? '';
</script>

<!-- Non-reactive attributes mirror providers that call setContext only once. -->
<div
	data-testid={providerName}
	data-ready={providerValue != null}
	data-resource-id={resourceId}
	data-error-ready={error != null}
	data-manager-ready={manager != null}
>
	<slot />
</div>
