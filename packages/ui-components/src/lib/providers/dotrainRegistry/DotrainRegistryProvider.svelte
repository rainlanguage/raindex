<script lang="ts">
	import type { DotrainRegistry } from '@rainlanguage/raindex';
	import type { RegistryManager } from '../registry/RegistryManager';
	import { setDotrainRegistryContext } from './context';

	export let registry: DotrainRegistry | null = null;
	export let error: string | undefined;
	export let manager: RegistryManager;

	$: setDotrainRegistryContext({ registry, error, manager });
</script>

<slot />
