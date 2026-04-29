<script lang="ts">
	import type { RaindexOrder } from '@rainlanguage/raindex';
	import CodeMirror from 'svelte-codemirror-editor';
	import { RainlangLR } from 'codemirror-rainlang';

	export let order: RaindexOrder | undefined = undefined;
	export let rainlangText: string | undefined = undefined;
	export let codeMirrorTheme;
	export let codeMirrorDisabled = true;
	export let codeMirrorStyles = {};
</script>

{#if rainlangText || order?.rainlang}
	<CodeMirror
		value={rainlangText || order?.rainlang}
		extensions={[RainlangLR]}
		theme={codeMirrorTheme}
		readonly={codeMirrorDisabled}
		useTab={true}
		tabSize={2}
		styles={{
			'&': {
				width: '100%'
			},
			...codeMirrorStyles
		}}
	/>
{/if}
