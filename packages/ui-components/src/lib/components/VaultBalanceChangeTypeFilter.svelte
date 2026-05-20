<script lang="ts">
	import { type VaultBalanceChangeFilter } from '@rainlanguage/raindex';
	import DropdownCheckbox from './dropdown/DropdownCheckbox.svelte';
	import { VAULT_BALANCE_CHANGE_FILTER_LABELS } from '../utils/vaultBalanceChangeLabels';

	export let value: VaultBalanceChangeFilter[] | undefined = undefined;

	const filterOptions = VAULT_BALANCE_CHANGE_FILTER_LABELS;

	let typeFilter: Partial<Record<VaultBalanceChangeFilter, string>> = {};

	$: {
		const keys = Object.keys(typeFilter) as VaultBalanceChangeFilter[];
		if (keys.length > 0 && keys.length < Object.keys(filterOptions).length) {
			value = keys.sort();
		} else {
			value = undefined;
		}
	}
</script>

<DropdownCheckbox
	options={filterOptions}
	bind:value={typeFilter}
	label="Change Type"
	allLabel="All types"
	onlyTitle={true}
/>
