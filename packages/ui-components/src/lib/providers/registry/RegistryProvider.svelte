<script lang="ts">
	import { setRegistryContext } from './context';
	import type { RegistryManager } from './RegistryManager';

	export let registryManager: RegistryManager;

	setRegistryContext(registryManager);
</script>

<slot />
