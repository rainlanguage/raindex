<script lang="ts">
	import '../app.css';
	import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { colorTheme } from '$lib/darkMode';
	import { browser } from '$app/environment';
	import { page } from '$app/stores';
	import Homepage from '$lib/components/Homepage.svelte';
	import LoadingWrapper from '$lib/components/LoadingWrapper.svelte';
	import {
		ToastProvider,
		WalletProvider,
		FixedBottomTransaction,
		RaindexClientProvider,
		LocalDbProvider,
		DotrainRegistryProvider,
		RegistryManager
	} from '@rainlanguage/ui-components';
	import { signerAddress } from '$lib/stores/wagmi';
	import { validChainIds } from '$lib/stores/settings';
	import ErrorPage from '$lib/components/ErrorPage.svelte';
	import TransactionProviderWrapper from '$lib/components/TransactionProviderWrapper.svelte';
	import { initWallet } from '$lib/services/handleWalletInitialization';
	import { REGISTRY_URL } from '$lib/constants';
	import { onMount } from 'svelte';
	import type { DotrainRegistry, RaindexClient } from '@rainlanguage/raindex';
	import { seedLocalDbSyncSnapshot } from '$lib/stores/localDbStatus';
	import LocalDbSyncGate from '$lib/components/LocalDbSyncGate.svelte';
	import {
		initializeSnapshotPoc,
		type SnapshotBootstrap,
		type SnapshotPocRuntime
	} from '$lib/services/snapshotPoc';

	const pageData = $page.data;
	let errorMessage = pageData.errorMessage;
	let localDb = pageData.localDb;
	let raindexClient = pageData.raindexClient;
	let registry = pageData.registry as DotrainRegistry | null;
	let snapshotBootstrap: SnapshotBootstrap | undefined;
	// POC mode is selected by a direct page load and remains fixed for this
	// document. The universal load keeps it enabled across client navigation.
	const snapshotPocEnabled = pageData.snapshotPocEnabled && $page.url.pathname !== '/';
	let snapshotBootstrapPending = snapshotPocEnabled && !errorMessage;
	let snapshotBootstrapPhase = 'Preparing local database';
	let snapshotBootstrapElapsed = 0;
	const registryManager = new RegistryManager(REGISTRY_URL);

	const queryClient = new QueryClient({
		defaultOptions: {
			queries: {
				staleTime: Infinity
			}
		}
	});

	let walletInitError: string | null = null;

	const initializeClientState = (client: RaindexClient) => {
		const uniqueChainIds = client.getUniqueChainIds();
		if (!uniqueChainIds.error) {
			validChainIds.set(uniqueChainIds.value);
		}

		client
			.getLocalDbSyncSnapshot()
			.then((snapshotResult) => {
				if (!snapshotResult.error) {
					seedLocalDbSyncSnapshot(snapshotResult.value);
				}
			})
			.catch(() => {
				// The live status callback will continue to update the sidebar and data gate.
			});
	};

	const freeSnapshotRuntime = (runtime: SnapshotPocRuntime) => {
		runtime.raindexClient.free();
		runtime.localDb.free();
		runtime.registry.free();
	};

	onMount(() => {
		if (!browser) return;

		if (snapshotPocEnabled) {
			let disposed = false;
			let ownedRuntime: SnapshotPocRuntime | undefined;
			const startedAt = performance.now();
			const elapsedTimer = window.setInterval(() => {
				snapshotBootstrapElapsed = (performance.now() - startedAt) / 1000;
			}, 100);

			initializeSnapshotPoc(pageData.registryUrl, (phase) => {
				if (!disposed) snapshotBootstrapPhase = phase;
			})
				.then((runtime) => {
					if (disposed) {
						freeSnapshotRuntime(runtime);
						return;
					}
					ownedRuntime = runtime;
					registry = runtime.registry;
					localDb = runtime.localDb;
					raindexClient = runtime.raindexClient;
					snapshotBootstrap = runtime.snapshotBootstrap;
					snapshotBootstrapPending = false;
					initializeClientState(runtime.raindexClient);
				})
				.catch((error: unknown) => {
					if (disposed) return;
					const message = (error as Error).message;
					errorMessage = 'Error initializing local database: ' + message;
					snapshotBootstrap = {
						status: 'failed',
						elapsedMs: performance.now() - startedAt,
						message
					};
					snapshotBootstrapPending = false;
				})
				.finally(() => window.clearInterval(elapsedTimer));

			return () => {
				disposed = true;
				window.clearInterval(elapsedTimer);
				if (ownedRuntime) freeSnapshotRuntime(ownedRuntime);
			};
		}

		if (!registry) return;
		if (raindexClient) initializeClientState(raindexClient as RaindexClient);
	});

	$: if (browser && window.navigator) {
		initWallet().then((error) => {
			walletInitError = error;
		});
	}
</script>

{#if snapshotBootstrap && snapshotBootstrap.status !== 'failed'}
	<div
		data-testid="snapshot-poc-result"
		role="status"
		aria-live="polite"
		class="fixed right-4 top-4 z-[110] max-w-md rounded-lg bg-emerald-700 px-5 py-3 text-sm text-white shadow-lg"
	>
		<div class="font-semibold">SQLite snapshot POC: {snapshotBootstrap.status}</div>
		<div>
			{#if snapshotBootstrap.status === 'installed'}
				{(snapshotBootstrap.bytesWritten / 1_000_000).toFixed(1)} MB installed from Cloudflare R2 in
				{(snapshotBootstrap.elapsedMs / 1000).toFixed(1)}s; database ready in
				{(snapshotBootstrap.readyElapsedMs / 1000).toFixed(1)}s
			{:else}
				Existing {(snapshotBootstrap.bytesWritten / 1_000_000).toFixed(1)} MB local snapshot opened in
				{snapshotBootstrap.elapsedMs.toFixed(0)}ms; database ready in
				{snapshotBootstrap.readyElapsedMs.toFixed(0)}ms
			{/if}
		</div>
	</div>
{/if}

{#if walletInitError}
	<div
		class="fixed bottom-4 left-1/2 z-[100] -translate-x-1/2 transform rounded-lg bg-red-500 px-6 py-3 text-white shadow-md"
	>
		{walletInitError}
	</div>
{/if}

<ToastProvider>
	<WalletProvider account={signerAddress}>
		<QueryClientProvider client={queryClient}>
			<TransactionProviderWrapper>
				<LoadingWrapper>
					{#if $page.url.pathname === '/'}
						<Homepage {colorTheme} />
					{:else if snapshotBootstrapPending}
						<div
							data-testid="snapshot-poc-loading-shell"
							class="flex h-screen w-full justify-start overflow-hidden bg-white dark:bg-gray-900 dark:text-gray-400"
						>
							<Sidebar {colorTheme} page={$page} localDbStatusOverride="syncing" />
							<p class="sr-only" role="status" aria-live="polite">
								{snapshotBootstrapPhase}
							</p>
							<main class="mx-auto h-screen w-full grow overflow-auto px-4 pt-14 lg:ml-64 lg:p-8">
								<LocalDbSyncGate
									syncingOverride={`${snapshotBootstrapPhase} (${snapshotBootstrapElapsed.toFixed(1)}s)`}
								/>
							</main>
						</div>
					{:else if errorMessage}
						<ErrorPage {errorMessage} />
					{:else}
						<DotrainRegistryProvider {registry} error={errorMessage} manager={registryManager}>
							<LocalDbProvider {localDb}>
								<RaindexClientProvider {raindexClient}>
									<div
										data-testid="layout-container"
										class={snapshotPocEnabled
											? 'flex h-screen w-full justify-start overflow-hidden bg-white dark:bg-gray-900 dark:text-gray-400'
											: 'flex min-h-screen w-full justify-start bg-white dark:bg-gray-900 dark:text-gray-400'}
									>
										<Sidebar {colorTheme} page={$page} />
										<main
											class={snapshotPocEnabled
												? 'mx-auto h-screen w-full grow overflow-auto px-4 pt-14 lg:ml-64 lg:p-8'
												: 'mx-auto h-full w-full grow overflow-x-auto px-4 pt-14 lg:ml-64 lg:p-8'}
										>
											<slot />
										</main>
									</div>
								</RaindexClientProvider>
							</LocalDbProvider>
						</DotrainRegistryProvider>
					{/if}
					<FixedBottomTransaction />
				</LoadingWrapper>
			</TransactionProviderWrapper>
		</QueryClientProvider>
	</WalletProvider>
</ToastProvider>
