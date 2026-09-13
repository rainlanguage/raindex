<script lang="ts">
	import { page } from '$app/stores';

	export let errorMessage: string | undefined = undefined;
	$: displayedError = errorMessage ?? $page.data?.errorMessage;
</script>

<div
	role="alert"
	data-testid="error-page"
	class="error-container mx-auto my-10 flex max-w-xl flex-col gap-4 p-2 text-center"
>
	<h1 class="text-2xl font-bold text-red-600">An Error Occurred</h1>
	<p>
		We encountered a problem loading the application configuration. Please try again later or
		<a
			href="https://t.me/rainlanguage"
			rel="noopener noreferrer"
			target="_blank"
			class="text-blue-500 hover:underline">contact support</a
		>.
	</p>

	{#if displayedError}
		<div class="error-details rounded border border-red-400 bg-red-100 p-4 text-left text-red-700">
			<h2 class="font-semibold">Error Details:</h2>
			<pre class="whitespace-pre-wrap break-words">{displayedError}</pre>
		</div>
	{/if}

	<a href="/" class="text-blue-500 hover:underline">Go to Homepage</a>
</div>
