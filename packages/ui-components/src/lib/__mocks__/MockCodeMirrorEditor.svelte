<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	export let value: string | undefined = undefined;

	const dispatch = createEventDispatcher();
</script>

<!-- Test double for `svelte-codemirror-editor`: renders all props as attributes
(like the shared MockComponent) and additionally forwards a native blur on the
element as the component-level `blur` event that CodeMirrorRainlang listens for. -->
<div
	data-testid="mock-component"
	{...$$props}
	tabindex="0"
	role="textbox"
	on:blur={() => dispatch('blur')}
>
	{value}
</div>
