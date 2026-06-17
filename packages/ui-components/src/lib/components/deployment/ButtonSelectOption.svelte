<script lang="ts">
	import { Button } from 'flowbite-svelte';
	export let active: boolean;
	export let buttonText: string;
	export let clickHandler: () => void | Promise<void>;
	export let dataTestId: string;
</script>

<Button
	size="lg"
	color="alternative"
	class={active ? 'border-2 border-blue-300 dark:border-blue-700' : 'border-2 border-transparent'}
	on:click={clickHandler}
	data-testid={dataTestId}
>
	{buttonText}
</Button>
