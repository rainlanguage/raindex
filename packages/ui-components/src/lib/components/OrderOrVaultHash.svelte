<script lang="ts">
	import { Button } from 'flowbite-svelte';
	import Hash, { HashType } from './Hash.svelte';
	import type {
		Address,
		RaindexOrder,
		RaindexOrderAsIO,
		RaindexVault
	} from '@rainlanguage/raindex';
	import {
		constructHashLink,
		isOrderOrVaultActive,
		extractHash
	} from '$lib/utils/constructHashLink';

	type OrderOrVault = RaindexOrder | RaindexOrderAsIO | RaindexVault;

	export let orderOrVault: OrderOrVault;
	export let type: 'orders' | 'vaults';
	export let chainId: number;
	export let raindexAddress: Address;

	$: hash = extractHash(orderOrVault);
	$: isActive = isOrderOrVaultActive(orderOrVault);
	$: linkPath = constructHashLink(orderOrVault, type, chainId, raindexAddress);
</script>

<a data-testid="order-or-vault-hash" href={linkPath}>
	<Button
		class="mr-1 mt-1 px-2 py-1 text-sm"
		color={isActive ? 'green' : 'yellow'}
		data-testid="vault-order-input"
		data-id={hash}><Hash type={HashType.Identifier} value={hash} copyOnClick={false} /></Button
	>
</a>
