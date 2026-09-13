import { render, waitFor, screen } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import Layout from './+layout.svelte';

const { mockPageStore, initialPageState, mockSignerAddressStore } = await vi.hoisted(
	() => import('$lib/__mocks__/stores')
);

const mockEnv = vi.hoisted(() => ({ browser: true }));
const mockInitWallet = vi.hoisted(() => vi.fn());
const mockInitializeSnapshotPoc = vi.hoisted(() => vi.fn());

vi.mock('$lib/services/handleWalletInitialization', () => ({
	initWallet: mockInitWallet
}));

vi.mock('$lib/services/snapshotPoc', () => ({
	initializeSnapshotPoc: mockInitializeSnapshotPoc
}));

vi.mock('$app/stores', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		page: mockPageStore
	};
});

vi.mock('$app/environment', () => mockEnv);

vi.mock('$lib/components/TransactionProviderWrapper.svelte', async () => {
	const MockComponent = (await import('$lib/__mocks__/MockComponent.svelte')).default;
	return {
		default: MockComponent
	};
});

vi.mock('$lib/components/Sidebar.svelte', async () => {
	const MockComponent = (await import('$lib/__mocks__/MockComponent.svelte')).default;
	return { default: MockComponent };
});

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	const MockComponent = (await import('$lib/__mocks__/MockComponent.svelte')).default;
	const ProviderProbe = (await import('$lib/__mocks__/ProviderProbe.svelte')).default;
	return {
		...(await importOriginal()),
		cachedWritableStore: vi.fn(),
		WalletProvider: MockComponent,
		ToastProvider: MockComponent,
		FixedBottomTransaction: MockComponent,
		DotrainRegistryProvider: ProviderProbe,
		LocalDbProvider: ProviderProbe,
		RaindexClientProvider: ProviderProbe
	};
});

vi.mock('$lib/stores/wagmi', () => ({
	signerAddress: mockSignerAddressStore
}));

vi.mock('$env/static/public', () => ({
	PUBLIC_WALLETCONNECT_PROJECT_ID: 'test-project-id'
}));

vi.mock('@wagmi/connectors', async (importOriginal) => {
	return {
		...(await importOriginal()),
		injected: vi.fn().mockReturnValue('injected-connector'),
		walletConnect: vi.fn().mockReturnValue('wallet-connect-connector')
	};
});

vi.mock('@tanstack/svelte-query', async (importOriginal) => {
	const MockComponent = (await import('$lib/__mocks__/MockComponent.svelte')).default;
	return {
		...(await importOriginal()),
		QueryClientProvider: MockComponent,
		QueryClient: vi.fn().mockImplementation(() => ({
			name: 'test'
		}))
	};
});

describe('Layout component', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetAllMocks();
		mockPageStore.reset();
		mockEnv.browser = true;
		mockInitWallet.mockResolvedValue(null);
	});

	it('displays an error message if wallet initialization fails', async () => {
		mockInitWallet.mockResolvedValue(
			'Failed to initialize wallet connection: Test error. Please try again or check console.'
		);
		mockPageStore.mockSetSubscribeValue(initialPageState);

		render(Layout);

		const errorMessage = await screen.findByText(
			'Failed to initialize wallet connection: Test error. Please try again or check console.'
		);
		expect(errorMessage).toBeInTheDocument();
	});

	it('renders Homepage when on root path', async () => {
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			url: new URL('http://localhost/')
		});

		const { container } = render(Layout);

		await waitFor(() => {
			expect(container.querySelector('main')).not.toBeInTheDocument();
			expect(screen.getByTestId('homepage')).toBeInTheDocument();
		});
	});

	it('does not bootstrap the snapshot on the provider-free homepage', async () => {
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			data: {
				...initialPageState.data,
				snapshotPocEnabled: true,
				registryUrl: 'https://registry.example'
			},
			url: new URL('http://localhost/?snapshot-poc=1')
		});

		render(Layout);

		await waitFor(() => expect(screen.getByTestId('homepage')).toBeInTheDocument());
		expect(mockInitializeSnapshotPoc).not.toHaveBeenCalled();
	});

	it('renders main content when not on root path', async () => {
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			url: new URL('http://localhost/some-page')
		});

		render(Layout);

		await waitFor(() => {
			expect(screen.getByTestId('layout-container')).toBeInTheDocument();
		});
	});

	it('renders snapshot progress immediately and then exposes the initialized providers', async () => {
		let resolveBootstrap: (runtime: unknown) => void = () => {};
		mockInitializeSnapshotPoc.mockImplementation(
			(_registryUrl: string, onPhase: (phase: string) => void) => {
				onPhase('Opening local database');
				return new Promise((resolve) => {
					resolveBootstrap = resolve;
				});
			}
		);
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			data: {
				...initialPageState.data,
				errorMessage: '',
				snapshotPocEnabled: true,
				registryUrl: 'https://registry.example',
				registry: null,
				localDb: null,
				raindexClient: null
			},
			url: new URL('http://localhost/orders?snapshot-poc=1')
		});

		render(Layout);

		expect(screen.getByTestId('snapshot-poc-loading-shell')).toBeInTheDocument();
		expect(screen.getByTestId('local-db-syncing-notice')).toHaveTextContent(
			'Opening local database'
		);
		expect(screen.getByRole('status')).toHaveTextContent('Opening local database');
		expect(mockInitializeSnapshotPoc).toHaveBeenCalledWith(
			'https://registry.example',
			expect.any(Function)
		);

		const raindexClient = {
			free: vi.fn(),
			getUniqueChainIds: () => ({ value: [], error: undefined }),
			getLocalDbSyncSnapshot: () =>
				Promise.resolve({ value: undefined, error: { readableMsg: 'not configured' } })
		};
		const registry = { id: 'registry', free: vi.fn() };
		const localDb = { id: 'database', free: vi.fn() };
		resolveBootstrap({
			registry,
			localDb,
			raindexClient,
			snapshotBootstrap: {
				status: 'reused',
				elapsedMs: 4,
				readyElapsedMs: 7,
				bytesWritten: 1_051_512_832
			}
		});

		await waitFor(() => {
			expect(screen.getByTestId('layout-container')).toBeInTheDocument();
			expect(screen.getByText('SQLite snapshot POC: reused')).toBeInTheDocument();
			expect(screen.getByTestId('snapshot-poc-result')).toHaveAttribute('role', 'status');
			expect(screen.getByTestId('registry-provider')).toHaveAttribute('data-ready', 'true');
			expect(screen.getByTestId('local-db-provider')).toHaveAttribute('data-ready', 'true');
			expect(screen.getByTestId('raindex-client-provider')).toHaveAttribute(
				'data-ready',
				'true'
			);
		});

		mockPageStore.mockSetSubscribeValue({
			data: {
				...initialPageState.data,
				snapshotPocEnabled: true,
				registryUrl: 'https://registry.example'
			},
			url: new URL('http://localhost/vaults')
		});

		await waitFor(() => expect(screen.getByTestId('layout-container')).toBeInTheDocument());
		expect(mockInitializeSnapshotPoc).toHaveBeenCalledTimes(1);
		expect(raindexClient.free).not.toHaveBeenCalled();
		expect(localDb.free).not.toHaveBeenCalled();
		expect(registry.free).not.toHaveBeenCalled();
	});

	it('shows the homepage immediately when navigating away during snapshot bootstrap', async () => {
		mockInitializeSnapshotPoc.mockImplementation(() => new Promise(() => {}));
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			data: {
				...initialPageState.data,
				snapshotPocEnabled: true,
				registryUrl: 'https://registry.example'
			},
			url: new URL('http://localhost/orders?snapshot-poc=1')
		});

		render(Layout);
		expect(screen.getByTestId('snapshot-poc-loading-shell')).toBeInTheDocument();

		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			data: {
				...initialPageState.data,
				snapshotPocEnabled: true,
				registryUrl: 'https://registry.example'
			},
			url: new URL('http://localhost/')
		});

		await waitFor(() => expect(screen.getByTestId('homepage')).toBeInTheDocument());
		expect(screen.queryByTestId('snapshot-poc-loading-shell')).not.toBeInTheDocument();
	});

	it('frees a snapshot runtime that finishes after the layout is destroyed', async () => {
		let resolveBootstrap: (runtime: unknown) => void = () => {};
		mockInitializeSnapshotPoc.mockImplementation(
			() =>
				new Promise((resolve) => {
					resolveBootstrap = resolve;
				})
		);
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			data: {
				...initialPageState.data,
				snapshotPocEnabled: true,
				registryUrl: 'https://registry.example'
			},
			url: new URL('http://localhost/orders?snapshot-poc=1')
		});
		const freeClient = vi.fn();
		const freeDatabase = vi.fn();
		const freeRegistry = vi.fn();

		const { unmount } = render(Layout);
		unmount();
		resolveBootstrap({
			registry: { free: freeRegistry },
			localDb: { free: freeDatabase },
			raindexClient: { free: freeClient }
		});

		await waitFor(() => {
			expect(freeClient).toHaveBeenCalledTimes(1);
			expect(freeDatabase).toHaveBeenCalledTimes(1);
			expect(freeRegistry).toHaveBeenCalledTimes(1);
		});
	});

	it('shows snapshot initialization errors in the error page', async () => {
		mockInitializeSnapshotPoc.mockRejectedValue(new Error('Snapshot SHA-256 mismatch'));
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			data: {
				...initialPageState.data,
				snapshotPocEnabled: true,
				registryUrl: 'https://registry.example'
			},
			url: new URL('http://localhost/orders?snapshot-poc=1')
		});

		render(Layout);

		await waitFor(() => {
			expect(screen.getByTestId('error-page')).toBeInTheDocument();
			expect(screen.queryByTestId('snapshot-poc-result')).not.toBeInTheDocument();
			expect(screen.getByTestId('error-page')).toHaveAttribute('role', 'alert');
			expect(
				screen.getByText('Error initializing local database: Snapshot SHA-256 mismatch')
			).toBeInTheDocument();
		});
	});

	it('does not initialize wallet when not in browser environment', async () => {
		const originalNavigator = global.navigator;
		mockEnv.browser = false;
		render(Layout);
		expect(mockInitWallet).not.toHaveBeenCalled();
		Object.defineProperty(global, 'navigator', {
			value: originalNavigator,
			writable: true
		});
	});

	it('displays an error page if page.error is set', async () => {
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			data: {
				...initialPageState.data,
				errorMessage: 'Test error'
			}
		});
		render(Layout);

		await waitFor(() => {
			expect(screen.getByText('Test error')).toBeInTheDocument();
			expect(screen.getByTestId('error-page')).toBeInTheDocument();
		});
	});
});
