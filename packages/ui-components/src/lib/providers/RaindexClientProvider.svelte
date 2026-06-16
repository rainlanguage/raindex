<script lang="ts">
	import { setContext } from 'svelte';
	import { RaindexClient } from '@rainlanguage/raindex';
	import { RAINDEX_CLIENT_CONTEXT_KEY } from '../hooks/useRaindexClient';

	export let raindexClient: RaindexClient;

	setContext(RAINDEX_CLIENT_CONTEXT_KEY, raindexClient);
</script>

<slot />
