import { describe, it, expect, vi, beforeEach } from 'vitest';
import { handleAddOrder } from '../lib/services/handleAddOrder';
import type { HandleAddOrderDependencies } from '../lib/services/handleAddOrder';
import type {
	DeploymentTransactionArgs,
	RaindexOrderBuilder,
	RaindexClient
} from '@rainlanguage/orderbook';
import type { TransactionManager } from '@rainlanguage/ui-components';
import { QKEY_ORDERS } from '@rainlanguage/ui-components';
import type { Hex } from 'viem';

// Mocks
const mockHandleTransactionConfirmationModal = vi.fn().mockResolvedValue({ success: true });
const mockErrToast = vi.fn();
const mockCreateApprovalTransaction = vi.fn();
const mockCreateAddOrderTransaction = vi.fn();
const mockCreateMetaTransaction = vi.fn();

const mockManager = {
	createApprovalTransaction: mockCreateApprovalTransaction,
	createAddOrderTransaction: mockCreateAddOrderTransaction,
	createMetaTransaction: mockCreateMetaTransaction
} as unknown as TransactionManager;

// New Mocks for builder
const mockGetDeploymentTransactionArgs = vi.fn();

const MOCKED_ACCOUNT = '0xmockAccount' as Hex;

const mockBuilder = {
	getDeploymentTransactionArgs: mockGetDeploymentTransactionArgs
} as unknown as RaindexOrderBuilder;

const mockRaindexClient = {} as unknown as RaindexClient;

const mockDeps: HandleAddOrderDependencies = {
	handleTransactionConfirmationModal: mockHandleTransactionConfirmationModal,
	errToast: mockErrToast,
	manager: mockManager,
	builder: mockBuilder,
	account: MOCKED_ACCOUNT,
	raindexClient: mockRaindexClient
};

const mockMetaCall = {
	to: '0x0000000000000000000000000000000000000000' as Hex,
	calldata: '0xdeadbeef' as Hex
};

const mockDeploymentArgs: DeploymentTransactionArgs = {
	approvals: [],
	deploymentCalldata: '0xdeploymentCalldata' as Hex,
	orderbookAddress: '0xorderbookAddressFromArgs' as Hex,
	chainId: 1,
	emitMetaCall: undefined
};

describe('handleAddOrder', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		// Reset to a simple mock, specific onConfirm logic will be handled in tests
		mockHandleTransactionConfirmationModal.mockReset();
		// Default to success for sequential calls
		mockHandleTransactionConfirmationModal.mockResolvedValue({ success: true });
		// mockGetDeploymentTransactionArgs will be reset/set in each test
	});

	it('should handle an order with no approvals, calling createAddOrderTransaction on confirm', async () => {
		const currentTestSpecificArgs: DeploymentTransactionArgs = {
			...mockDeploymentArgs,
			approvals: [],
			emitMetaCall: mockMetaCall
		};
		mockGetDeploymentTransactionArgs.mockResolvedValue({
			value: currentTestSpecificArgs,
			error: null
		});

		await handleAddOrder(mockDeps);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(2);
		expect(mockCreateApprovalTransaction).not.toHaveBeenCalled();

		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			1,
			expect.objectContaining({
				modalTitle: 'Publishing metadata',
				args: expect.objectContaining({
					toAddress: mockMetaCall.to,
					chainId: currentTestSpecificArgs.chainId,
					calldata: mockMetaCall.calldata,
					onConfirm: expect.any(Function)
				})
			})
		);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			2,
			expect.objectContaining({
				open: true,
				modalTitle: 'Deploying your order',
				args: expect.objectContaining({
					toAddress: currentTestSpecificArgs.orderbookAddress,
					chainId: currentTestSpecificArgs.chainId,
					calldata: currentTestSpecificArgs.deploymentCalldata,
					onConfirm: expect.any(Function)
				})
			})
		);

		// Manually call the onConfirm callbacks
		const metaOnConfirm = mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
		await metaOnConfirm('0xmetaTxHash' as Hex);
		expect(mockCreateMetaTransaction).toHaveBeenCalledTimes(1);
		expect(mockCreateMetaTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: '0xmetaTxHash',
				queryKey: QKEY_ORDERS
			})
		);

		const addOrderOnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[1][0].args.onConfirm;
		const mockAddOrderTxHash = '0xaddOrderHashFromTest' as Hex;
		await addOrderOnConfirm(mockAddOrderTxHash);

		expect(mockCreateAddOrderTransaction).toHaveBeenCalledTimes(1);
		expect(mockCreateAddOrderTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: mockAddOrderTxHash, // Check for the specific hash
				queryKey: QKEY_ORDERS,
				raindexClient: mockRaindexClient
			})
		);
	});

	it('should handle an order with one approval, then add order, calling respective transaction creations on confirm', async () => {
		const approval1 = {
			token: '0xtoken1' as Hex,
			calldata: '0xapprovalcalldata1' as Hex,
			symbol: 'TKN1'
		};
		const currentTestSpecificArgs: DeploymentTransactionArgs = {
			...mockDeploymentArgs,
			approvals: [approval1],
			emitMetaCall: mockMetaCall
		};
		mockGetDeploymentTransactionArgs.mockResolvedValue({
			value: currentTestSpecificArgs,
			error: null
		});

		// Mock sequential calls - approval first, then deployment
		mockHandleTransactionConfirmationModal
			.mockResolvedValueOnce({ success: true, hash: '0xapprovalHash' })
			.mockResolvedValueOnce({ success: true, hash: '0xabc123' })
			.mockResolvedValueOnce({ success: true, hash: '0xdeploymentHash' });

		await handleAddOrder(mockDeps);

		// Should be called three times: approval, metadata, deployment
		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(3);

		// First call should be for approval
		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			1,
			expect.objectContaining({
				open: true,
				modalTitle: 'Approving TKN1 spend',
				closeOnConfirm: true,
				args: expect.objectContaining({
					toAddress: approval1.token,
					chainId: currentTestSpecificArgs.chainId,
					calldata: approval1.calldata,
					onConfirm: expect.any(Function)
				})
			})
		);

		// Second call should be for metadata
		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			2,
			expect.objectContaining({
				open: true,
				modalTitle: 'Publishing metadata',
				args: expect.objectContaining({
					toAddress: mockMetaCall.to,
					chainId: currentTestSpecificArgs.chainId,
					calldata: mockMetaCall.calldata,
					onConfirm: expect.any(Function)
				})
			})
		);

		// Third call should be for deployment
		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			3,
			expect.objectContaining({
				open: true,
				modalTitle: 'Deploying your order',
				args: expect.objectContaining({
					toAddress: currentTestSpecificArgs.orderbookAddress,
					chainId: currentTestSpecificArgs.chainId,
					calldata: currentTestSpecificArgs.deploymentCalldata,
					onConfirm: expect.any(Function)
				})
			})
		);

		// Verify onConfirm functions would call the right transaction managers
		const approvalOnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
		const metaOnConfirm = mockHandleTransactionConfirmationModal.mock.calls[1][0].args.onConfirm;
		const deploymentOnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[2][0].args.onConfirm;

		// Simulate calling onConfirm for approval
		await approvalOnConfirm('0xapprovalTxHash' as Hex);
		expect(mockCreateApprovalTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: '0xapprovalTxHash',
				queryKey: QKEY_ORDERS
			})
		);

		// Simulate calling onConfirm for metadata
		await metaOnConfirm('0xabc123' as Hex);
		expect(mockCreateMetaTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: '0xabc123',
				queryKey: QKEY_ORDERS
			})
		);

		// Simulate calling onConfirm for deployment
		await deploymentOnConfirm('0xdeploymentTxHash' as Hex);
		expect(mockCreateAddOrderTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: '0xdeploymentTxHash',
				queryKey: QKEY_ORDERS,
				raindexClient: mockRaindexClient
			})
		);
	});

	it('should handle an order with multiple approvals, then add order, calling respective transaction creations on confirm', async () => {
		const approval1 = {
			token: '0xtoken1' as Hex,
			calldata: '0xapprovalcalldata1' as Hex,
			symbol: 'TKN1'
		};
		const approval2 = {
			token: '0xtoken2' as Hex,
			calldata: '0xapprovalcalldata2' as Hex,
			symbol: 'TKN2'
		};
		const currentTestSpecificArgs: DeploymentTransactionArgs = {
			...mockDeploymentArgs,
			approvals: [approval1, approval2],
			emitMetaCall: mockMetaCall
		};
		mockGetDeploymentTransactionArgs.mockResolvedValue({
			value: currentTestSpecificArgs,
			error: null
		});

		// Mock sequential calls - 2 approvals + metadata + deployment
		mockHandleTransactionConfirmationModal
			.mockResolvedValueOnce({ success: true, hash: '0xapproval1Hash' })
			.mockResolvedValueOnce({ success: true, hash: '0xapproval2Hash' })
			.mockResolvedValueOnce({ success: true, hash: '0xdeploymentHash' });

		await handleAddOrder(mockDeps);

		// Should be called 4 times: 2 approvals + metadata + deployment
		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(4);

		// First approval
		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			1,
			expect.objectContaining({
				modalTitle: 'Approving TKN1 spend',
				args: expect.objectContaining({ toAddress: approval1.token })
			})
		);

		// Second approval
		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			2,
			expect.objectContaining({
				modalTitle: 'Approving TKN2 spend',
				args: expect.objectContaining({ toAddress: approval2.token })
			})
		);

		// Metadata
		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			3,
			expect.objectContaining({
				modalTitle: 'Publishing metadata',
				args: expect.objectContaining({ toAddress: mockMetaCall.to })
			})
		);

		// Deployment
		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			4,
			expect.objectContaining({
				modalTitle: 'Deploying your order',
				args: expect.objectContaining({ toAddress: currentTestSpecificArgs.orderbookAddress })
			})
		);

		// Verify onConfirm functions would call the right transaction managers
		const approval1OnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
		const approval2OnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[1][0].args.onConfirm;
		const metaOnConfirm = mockHandleTransactionConfirmationModal.mock.calls[2][0].args.onConfirm;
		const deploymentOnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[3][0].args.onConfirm;

		// Simulate calling onConfirm for first approval
		await approval1OnConfirm('0xapproval1TxHash' as Hex);
		expect(mockCreateApprovalTransaction).toHaveBeenNthCalledWith(
			1,
			expect.objectContaining({ txHash: '0xapproval1TxHash' })
		);

		// Simulate calling onConfirm for second approval
		await approval2OnConfirm('0xapproval2TxHash' as Hex);
		expect(mockCreateApprovalTransaction).toHaveBeenNthCalledWith(
			2,
			expect.objectContaining({ txHash: '0xapproval2TxHash' })
		);

		// Simulate calling onConfirm for metadata
		await metaOnConfirm('0xabc124' as Hex);
		expect(mockCreateMetaTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: '0xabc124',
				queryKey: QKEY_ORDERS
			})
		);

		// Simulate calling onConfirm for deployment
		await deploymentOnConfirm('0xdeploymentTxHash' as Hex);
		expect(mockCreateAddOrderTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				txHash: '0xdeploymentTxHash',
				raindexClient: mockRaindexClient
			})
		);
	});

	it('should call errToast and stop if metadata publication is cancelled or fails', async () => {
		const currentTestSpecificArgs: DeploymentTransactionArgs = {
			...mockDeploymentArgs,
			emitMetaCall: mockMetaCall
		};
		mockGetDeploymentTransactionArgs.mockResolvedValue({
			value: currentTestSpecificArgs,
			error: null
		});

		// First modal: metadata returns unsuccessfully
		mockHandleTransactionConfirmationModal.mockResolvedValueOnce({ success: false });

		await handleAddOrder(mockDeps);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(1);
		expect(mockErrToast).toHaveBeenCalledWith('Metadata publication was cancelled or failed');
		expect(mockCreateMetaTransaction).not.toHaveBeenCalled();
		expect(mockCreateAddOrderTransaction).not.toHaveBeenCalled();
	});

	it('should call errToast and stop if metadata publication throws', async () => {
		const currentTestSpecificArgs: DeploymentTransactionArgs = {
			...mockDeploymentArgs,
			emitMetaCall: mockMetaCall
		};
		mockGetDeploymentTransactionArgs.mockResolvedValue({
			value: currentTestSpecificArgs,
			error: null
		});

		const thrownError = new Error('metadata failed');
		mockHandleTransactionConfirmationModal.mockRejectedValueOnce(thrownError);

		await handleAddOrder(mockDeps);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(1);
		expect(mockErrToast).toHaveBeenCalledWith(
			`Metadata publication failed: ${thrownError.message}`
		);
		expect(mockCreateMetaTransaction).not.toHaveBeenCalled();
		expect(mockCreateAddOrderTransaction).not.toHaveBeenCalled();
	});

	it('should use different txHashes from modal onConfirms when they differ', async () => {
		const approval1 = {
			token: '0xtoken1' as Hex,
			calldata: '0xapprovalcalldata1' as Hex,
			symbol: 'TKN1'
		};
		const currentTestSpecificArgs: DeploymentTransactionArgs = {
			...mockDeploymentArgs,
			approvals: [approval1],
			emitMetaCall: mockMetaCall
		};
		mockGetDeploymentTransactionArgs.mockResolvedValue({
			value: currentTestSpecificArgs,
			error: null
		});

		// Mock sequential calls with specific hashes
		mockHandleTransactionConfirmationModal
			.mockResolvedValueOnce({ success: true, hash: '0xspecificApprovalHash' })
			.mockResolvedValueOnce({ success: true, hash: '0xabc125' })
			.mockResolvedValueOnce({ success: true, hash: '0xspecificAddOrderHash' });

		await handleAddOrder(mockDeps);

		// Should be called three times: approval + metadata + deployment
		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(3);

		// Verify onConfirm functions would call the right transaction managers with specific hashes
		const approvalOnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
		const metaOnConfirm = mockHandleTransactionConfirmationModal.mock.calls[1][0].args.onConfirm;
		const addOrderOnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[2][0].args.onConfirm;

		// Simulate calling onConfirm for approval with specific hash
		const specificApprovalHash = '0xspecificApprovalHash' as Hex;
		await approvalOnConfirm(specificApprovalHash);

		expect(mockCreateApprovalTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: specificApprovalHash, // Check specific hash
				queryKey: QKEY_ORDERS
			})
		);

		// Simulate calling onConfirm for metadata
		const metaHash = '0xabc125' as Hex;
		await metaOnConfirm(metaHash);
		expect(mockCreateMetaTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: metaHash,
				queryKey: QKEY_ORDERS
			})
		);

		// Simulate calling onConfirm for deployment with specific hash
		const specificAddOrderHash = '0xspecificAddOrderHash' as Hex;
		await addOrderOnConfirm(specificAddOrderHash);

		expect(mockCreateAddOrderTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: specificAddOrderHash, // Check specific hash
				queryKey: QKEY_ORDERS,
				raindexClient: mockRaindexClient
			})
		);
	});

	it('should call errToast if account is null', async () => {
		const depsWithNullAccount = { ...mockDeps, account: null };

		await handleAddOrder(depsWithNullAccount);

		expect(mockErrToast).toHaveBeenCalledWith('Could not deploy: No wallet address found');
		expect(mockGetDeploymentTransactionArgs).not.toHaveBeenCalled();
		expect(mockHandleTransactionConfirmationModal).not.toHaveBeenCalled();
		expect(mockCreateApprovalTransaction).not.toHaveBeenCalled();
		expect(mockCreateAddOrderTransaction).not.toHaveBeenCalled();
	});

	it('should call errToast if getDeploymentTransactionArgs returns an error', async () => {
		const customErrorMsg = 'Custom error from builder';
		mockGetDeploymentTransactionArgs.mockResolvedValue({
			value: null,
			error: { msg: customErrorMsg }
		});

		await handleAddOrder(mockDeps);

		expect(mockErrToast).toHaveBeenCalledWith(`Could not deploy: ${customErrorMsg}`);
		expect(mockHandleTransactionConfirmationModal).not.toHaveBeenCalled();
		expect(mockCreateApprovalTransaction).not.toHaveBeenCalled();
		expect(mockCreateAddOrderTransaction).not.toHaveBeenCalled();
	});

	it('should call errToast and stop if approval fails', async () => {
		const approval1 = {
			token: '0xtoken1' as Hex,
			calldata: '0xapprovalcalldata1' as Hex,
			symbol: 'TKN1'
		};
		const currentTestSpecificArgs: DeploymentTransactionArgs = {
			...mockDeploymentArgs,
			approvals: [approval1]
		};
		mockGetDeploymentTransactionArgs.mockResolvedValue({
			value: currentTestSpecificArgs,
			error: null
		});

		// Mock approval failure - first call fails, second should not happen
		mockHandleTransactionConfirmationModal.mockResolvedValueOnce({ success: false }); // Approval fails

		await handleAddOrder(mockDeps);

		// Should only be called once for the failed approval
		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(1);
		expect(mockErrToast).toHaveBeenCalledWith('Approval was cancelled or failed');
		expect(mockCreateApprovalTransaction).not.toHaveBeenCalled();
		expect(mockCreateAddOrderTransaction).not.toHaveBeenCalled();
	});
});
