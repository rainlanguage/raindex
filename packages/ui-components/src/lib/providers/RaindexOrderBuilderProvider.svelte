<script lang="ts">
	import { setContext } from 'svelte';
	import { RaindexOrderBuilder } from '@rainlanguage/orderbook';
	import { RAINDEX_ORDER_BUILDER_CONTEXT_KEY } from '../hooks/useRaindexOrderBuilder';

	export let builder: RaindexOrderBuilder;

	setContext(RAINDEX_ORDER_BUILDER_CONTEXT_KEY, builder);
</script>

<slot />
