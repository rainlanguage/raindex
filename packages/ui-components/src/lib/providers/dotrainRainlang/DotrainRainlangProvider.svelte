<script lang="ts">
	import type { DotrainRainlang } from '@rainlanguage/orderbook';
	import type { RainlangManager } from '../rainlang/RainlangManager';
	import { setDotrainRainlangContext } from './context';

	export let rainlang: DotrainRainlang | null = null;
	export let error: string | undefined;
	export let manager: RainlangManager;

	$: setDotrainRainlangContext({ rainlang, error, manager });
</script>

<slot />
