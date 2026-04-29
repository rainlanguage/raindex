import { describe, it, expect, vi, beforeEach } from 'vitest';
import { TransactionStore } from '../lib/models/Transaction';
import {
	TransactionStatusMessage,
	TransactionStoreErrorMessage,
	type TransactionArgs,
	TransactionName,
	type IndexingContext,
	type AwaitIndexingFn
} from '../lib/types/transaction';
import { waitForTransactionReceipt, type Config } from '@wagmi/core';
import { get } from 'svelte/store';
import type { Chain } from 'viem';
import type { ToastLink } from '../lib/types/toast';
import type { SgVault } from '@rainlanguage/raindex';

vi.mock('@wagmi/core', () => ({
	waitForTransactionReceipt: vi.fn()
}));

describe('TransactionStore', () => {
	const mockChain: Chain = {
		id: 1,
		name: 'Ethereum',
		nativeCurrency: {
			name: 'Ether',
			symbol: 'ETH',
			decimals: 18
		},
		rpcUrls: {
			default: { http: ['https://eth.llamarpc.com'] },
			public: { http: ['https://eth.llamarpc.com'] }
		},
		blockExplorers: {
			default: { name: 'Etherscan', url: 'https://etherscan.io' }
		}
	};

	const mockConfig = {
		chains: [mockChain] as const,
		connectors: [],
		storage: {
			getItem: vi.fn(),
			setItem: vi.fn(),
			removeItem: vi.fn(),
			key: 'wagmi'
		},
		state: {
			connections: new Map(),
			status: 'connected',
			current: undefined
		},
		setState: vi.fn(),
		subscribe: vi.fn(),
		destroy: vi.fn(),
		getClient: vi.fn(),
		_internal: {}
	} as unknown as Config;

	const mockVault: SgVault = {
		id: 'vault1',
		vaultId: 'vault1',
		token: {
			id: 'token1',
			address: '0xTokenAddress1',
			name: 'Token1',
			symbol: 'TKN1',
			decimals: '18'
		},
		owner: '0xOwnerAddress',
		ordersAsInput: [],
		ordersAsOutput: [],
		balanceChanges: [],
		balance: '1000000000000000000',
		raindex: {
			id: '0x00'
		}
	};

	const mockChainId = 1;
	const mockTxHash = '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef';
	const mockOrderHash = '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890';
	const mockOnSuccess = vi.fn();
	const mockOnError = vi.fn();

	const mockToastLinks: ToastLink[] = [
		{
			link: 'https://etherscan.io/tx/test-tx',
			label: 'View on Explorer'
		}
	];

	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('without awaitIndexingFn', () => {
		it('should initialize with IDLE status and correct links', () => {
			const transaction = new TransactionStore(
				{
					config: mockConfig,
					chainId: mockChainId,
					txHash: mockTxHash,
					orderHash: mockOrderHash,
					name: TransactionName.APPROVAL,
					errorMessage: 'Transaction failed',
					successMessage: 'Transaction successful',
					queryKey: 'approval',
					toastLinks: mockToastLinks,
					entity: mockVault
				} as TransactionArgs & { config: Config },
				mockOnSuccess,
				mockOnError
			);

			const state = get(transaction.state);
			expect(state.status).toBe(TransactionStatusMessage.IDLE);
			expect(state.links).toEqual(mockToastLinks);
		});

		it('should mark SUCCESS immediately after receipt when no indexing function', async () => {
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			vi.mocked(waitForTransactionReceipt).mockResolvedValue({} as any);

			const transaction = new TransactionStore(
				{
					config: mockConfig,
					chainId: mockChainId,
					txHash: mockTxHash,
					name: TransactionName.APPROVAL,
					errorMessage: 'Transaction failed',
					successMessage: 'Transaction successful',
					queryKey: 'approval',
					toastLinks: mockToastLinks
				} as TransactionArgs & { config: Config },
				mockOnSuccess,
				mockOnError
			);

			await transaction.execute();

			const state = get(transaction.state);
			expect(state.status).toBe(TransactionStatusMessage.SUCCESS);
			expect(mockOnSuccess).toHaveBeenCalled();
			expect(mockOnError).not.toHaveBeenCalled();
		});

		it('should handle transaction receipt failure', async () => {
			vi.mocked(waitForTransactionReceipt).mockRejectedValue(new Error('Transaction failed'));

			const transaction = new TransactionStore(
				{
					config: mockConfig,
					chainId: mockChainId,
					txHash: mockTxHash,
					name: TransactionName.APPROVAL,
					errorMessage: 'Transaction failed',
					successMessage: 'Transaction successful',
					queryKey: 'approval',
					toastLinks: mockToastLinks
				} as TransactionArgs & { config: Config },
				mockOnSuccess,
				mockOnError
			);

			await transaction.execute();

			const state = get(transaction.state);
			expect(state.status).toBe(TransactionStatusMessage.ERROR);
			expect(state.errorDetails).toBe(TransactionStoreErrorMessage.RECEIPT_FAILED);
			expect(state.links).toEqual(mockToastLinks);
			expect(mockOnError).toHaveBeenCalled();
		});
	});

	describe('with awaitIndexingFn', () => {
		it('should call awaitIndexingFn after receipt and handle success', async () => {
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			vi.mocked(waitForTransactionReceipt).mockResolvedValue({} as any);

			const mockAwaitIndexingFn: AwaitIndexingFn = vi.fn(async (ctx: IndexingContext) => {
				ctx.updateState({ status: TransactionStatusMessage.SUCCESS });
				ctx.onSuccess();
			});

			const transaction = new TransactionStore(
				{
					config: mockConfig,
					chainId: mockChainId,
					txHash: mockTxHash,
					name: TransactionName.REMOVAL,
					errorMessage: 'Transaction failed',
					successMessage: 'Transaction successful',
					queryKey: 'removeOrder',
					toastLinks: mockToastLinks,
					awaitIndexingFn: mockAwaitIndexingFn
				} as TransactionArgs & { config: Config },
				mockOnSuccess,
				mockOnError
			);

			await transaction.execute();

			expect(mockAwaitIndexingFn).toHaveBeenCalledWith(
				expect.objectContaining({
					updateState: expect.any(Function),
					onSuccess: mockOnSuccess,
					onError: mockOnError,
					links: mockToastLinks
				})
			);

			const state = get(transaction.state);
			expect(state.status).toBe(TransactionStatusMessage.SUCCESS);
			expect(mockOnSuccess).toHaveBeenCalled();
		});

		it('should call awaitIndexingFn after receipt and handle error', async () => {
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			vi.mocked(waitForTransactionReceipt).mockResolvedValue({} as any);

			const mockAwaitIndexingFn: AwaitIndexingFn = vi.fn(async (ctx: IndexingContext) => {
				ctx.updateState({
					status: TransactionStatusMessage.ERROR,
					errorDetails: TransactionStoreErrorMessage.SUBGRAPH_FAILED
				});
				ctx.onError();
			});

			const transaction = new TransactionStore(
				{
					config: mockConfig,
					chainId: mockChainId,
					txHash: mockTxHash,
					name: TransactionName.REMOVAL,
					errorMessage: 'Transaction failed',
					successMessage: 'Transaction successful',
					queryKey: 'removeOrder',
					toastLinks: mockToastLinks,
					awaitIndexingFn: mockAwaitIndexingFn
				} as TransactionArgs & { config: Config },
				mockOnSuccess,
				mockOnError
			);

			await transaction.execute();

			const state = get(transaction.state);
			expect(state.status).toBe(TransactionStatusMessage.ERROR);
			expect(state.errorDetails).toBe(TransactionStoreErrorMessage.SUBGRAPH_FAILED);
			expect(mockOnError).toHaveBeenCalled();
		});

		it('should handle timeout error from indexing function', async () => {
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			vi.mocked(waitForTransactionReceipt).mockResolvedValue({} as any);

			const mockAwaitIndexingFn: AwaitIndexingFn = vi.fn(async (ctx: IndexingContext) => {
				ctx.updateState({
					status: TransactionStatusMessage.ERROR,
					errorDetails: TransactionStoreErrorMessage.SUBGRAPH_TIMEOUT_ERROR
				});
				ctx.onError();
			});

			const transaction = new TransactionStore(
				{
					config: mockConfig,
					chainId: mockChainId,
					txHash: mockTxHash,
					name: TransactionName.REMOVAL,
					errorMessage: 'Transaction failed',
					successMessage: 'Transaction successful',
					queryKey: 'removeOrder',
					toastLinks: mockToastLinks,
					awaitIndexingFn: mockAwaitIndexingFn
				} as TransactionArgs & { config: Config },
				mockOnSuccess,
				mockOnError
			);

			await transaction.execute();

			const state = get(transaction.state);
			expect(state.status).toBe(TransactionStatusMessage.ERROR);
			expect(state.errorDetails).toBe(TransactionStoreErrorMessage.SUBGRAPH_TIMEOUT_ERROR);
			expect(mockOnError).toHaveBeenCalled();
		});

		it('should allow indexing function to add links', async () => {
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			vi.mocked(waitForTransactionReceipt).mockResolvedValue({} as any);

			const newOrderHash = '0xneworderhash123';
			const mockAwaitIndexingFn: AwaitIndexingFn = vi.fn(async (ctx: IndexingContext) => {
				ctx.updateState({ status: TransactionStatusMessage.SUCCESS });
				// Add a "View order" link
				const newLink = {
					link: `/orders/1-0xraindex-${newOrderHash}`,
					label: 'View order'
				};
				ctx.updateState({ links: [newLink, ...ctx.links] });
				ctx.onSuccess();
			});

			const transaction = new TransactionStore(
				{
					config: mockConfig,
					chainId: mockChainId,
					txHash: mockTxHash,
					name: 'Deploying order',
					errorMessage: 'Deployment failed',
					successMessage: 'Order deployed successfully',
					queryKey: 'addOrder',
					toastLinks: mockToastLinks,
					awaitIndexingFn: mockAwaitIndexingFn
				} as TransactionArgs & { config: Config },
				mockOnSuccess,
				mockOnError
			);

			await transaction.execute();

			const state = get(transaction.state);
			expect(state.status).toBe(TransactionStatusMessage.SUCCESS);
			expect(state.links).toHaveLength(2);
			expect(state.links[0]).toEqual({
				link: `/orders/1-0xraindex-${newOrderHash}`,
				label: 'View order'
			});
			expect(mockOnSuccess).toHaveBeenCalled();
		});

		it('should update status to PENDING_SUBGRAPH when indexing function sets it', async () => {
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			vi.mocked(waitForTransactionReceipt).mockResolvedValue({} as any);

			let capturedCtx: IndexingContext | null = null;
			const mockAwaitIndexingFn: AwaitIndexingFn = vi.fn(async (ctx: IndexingContext) => {
				capturedCtx = ctx;
				ctx.updateState({ status: TransactionStatusMessage.PENDING_SUBGRAPH });
				// Simulate some async work
				await new Promise((resolve) => setTimeout(resolve, 10));
				ctx.updateState({ status: TransactionStatusMessage.SUCCESS });
				ctx.onSuccess();
			});

			const transaction = new TransactionStore(
				{
					config: mockConfig,
					chainId: mockChainId,
					txHash: mockTxHash,
					name: TransactionName.DEPOSIT,
					errorMessage: 'Deposit failed',
					successMessage: 'Deposit successful',
					queryKey: 'deposit',
					toastLinks: mockToastLinks,
					awaitIndexingFn: mockAwaitIndexingFn
				} as TransactionArgs & { config: Config },
				mockOnSuccess,
				mockOnError
			);

			await transaction.execute();

			expect(capturedCtx).not.toBeNull();
			expect(mockOnSuccess).toHaveBeenCalled();

			const state = get(transaction.state);
			expect(state.status).toBe(TransactionStatusMessage.SUCCESS);
		});
	});
});
