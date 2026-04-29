<script lang="ts">
	import { setContext } from 'svelte';
	import { DotrainOrderGui } from '@rainlanguage/raindex';
	import { GUI_CONTEXT_KEY } from '../hooks/useGui';

	export let gui: DotrainOrderGui;

	setContext(GUI_CONTEXT_KEY, gui);
</script>

<slot />
