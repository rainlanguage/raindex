<script lang="ts">
	import type { RaindexOrder } from '@rainlanguage/raindex';
	import CodeMirror from 'svelte-codemirror-editor';
	import { RainlangLR } from 'codemirror-rainlang';
	import { stripTrailingWhitespace } from '../utils/stripTrailingWhitespace';

	export let order: RaindexOrder | undefined = undefined;
	export let rainlangText: string | undefined = undefined;
	export let codeMirrorTheme;
	export let codeMirrorDisabled = true;
	export let codeMirrorStyles = {};
	export let onSave: ((text: string) => void) | undefined = undefined;

	// The text currently shown in the editor. Seeded from whichever source prop
	// is provided and kept up to date as the user edits via the change event.
	let value: string | undefined;
	$: value = rainlangText ?? order?.rainlang;

	// Strip trailing whitespace when the editor loses focus (save/blur) so the
	// persisted rainlang never carries dangling whitespace.
	function handleBlur() {
		if (value === undefined) return;
		const stripped = stripTrailingWhitespace(value);
		value = stripped;
		rainlangText = stripped;
		onSave?.(stripped);
	}
</script>

<!-- Render the editor whenever a source value is present, including an empty
string left by stripping whitespace-only input on blur. -->
{#if value != null}
	<CodeMirror
		{value}
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
		on:change={(e) => {
			value = e.detail;
		}}
		on:blur={handleBlur}
	/>
{/if}
