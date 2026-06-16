<script lang="ts">
	import { setRainlangContext } from './context';
	import type { RainlangManager } from './RainlangManager';

	export let rainlangManager: RainlangManager;

	setRainlangContext(rainlangManager);
</script>

<slot />
