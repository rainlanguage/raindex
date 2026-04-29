import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
	handleVaultDeposit,
	type VaultDepositHandlerDependencies
} from '../lib/services/handleVaultDeposit';
import { Float, type RaindexClient, type RaindexVault } from '@rainlanguage/raindex';
import type { Hex } from 'viem';
import { waitFor } from '@testing-library/svelte';
import type { TransactionManager } from '@rainlanguage/ui-components';

// Mocks
const mockHandleDepositModal = vi.fn();
const mockHandleTransactionConfirmationModal = vi.fn();
const mockErrToast = vi.fn();
const mockCreateDepositTransaction = vi.fn();
const mockCreateApprovalTransaction = vi.fn();

const mockManager = {
	createDepositTransaction: mockCreateDepositTransaction,
	createApprovalTransaction: mockCreateApprovalTransaction
};

const mockRaindexClient = {} as unknown as RaindexClient;

const mockVault = {
	id: '0xvaultid',
	token: {
		address: '0xtokenaddress',
		symbol: 'TEST'
	},
	getApprovalCalldata: vi.fn(),
	getDepositCalldata: vi.fn()
} as unknown as RaindexVault;

const mockDeps: VaultDepositHandlerDependencies = {
	raindexClient: mockRaindexClient,
	vault: mockVault,
	account: '0xaccount' as Hex,
	handleDepositModal: mockHandleDepositModal,
	handleTransactionConfirmationModal: mockHandleTransactionConfirmationModal,
	errToast: mockErrToast,
	manager: mockManager as unknown as TransactionManager
};

describe('handleVaultDeposit', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should call handleDepositModal with correct arguments', async () => {
		await handleVaultDeposit(mockDeps);
		expect(mockHandleDepositModal).toHaveBeenCalledWith({
			open: true,
			args: {
				vault: mockVault,
				account: mockDeps.account
			},
			onSubmit: expect.any(Function)
		});
	});

	describe('onSubmit callback from handleDepositModal', () => {
		const mockAmount = Float.parse('100').value as Float;
		const mockApprovalCalldata = '0xapprovalcalldata' as Hex;
		const mockDepositCalldata = '0xdepositcalldata' as Hex;
		const mockTxHashApproval = '0xtxhashapproval' as Hex;
		const mockTxHashDeposit = '0xtxhashdeposit' as Hex;

		beforeEach(async () => {
			await handleVaultDeposit(mockDeps);
		});

		it('should execute deposit directly if getVaultApprovalCalldata returns error', async () => {
			vi.mocked(mockVault.getApprovalCalldata).mockResolvedValue({
				error: { msg: 'Approval error', readableMsg: 'Approval error readable' },
				value: undefined
			});
			vi.mocked(mockVault.getDepositCalldata).mockResolvedValue({
				value: mockDepositCalldata,
				error: undefined
			});

			const onSubmitCall = mockHandleDepositModal.mock.calls[0][0].onSubmit;
			await onSubmitCall(mockAmount);

			expect(mockVault.getApprovalCalldata).toHaveBeenCalledWith(mockAmount);
			expect(mockErrToast).not.toHaveBeenCalledWith('Approval error');
			expect(mockVault.getDepositCalldata).toHaveBeenCalledWith(mockAmount);
			expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(1);
			expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledWith({
				open: true,
				modalTitle: 'Depositing 100 TEST',
				closeOnConfirm: false,
				args: expect.objectContaining({ calldata: mockDepositCalldata })
			});
		});

		it('should show error toast if getVaultDepositCalldata returns an error (direct deposit flow)', async () => {
			vi.mocked(mockVault.getApprovalCalldata).mockResolvedValue({
				error: { msg: 'Approval error', readableMsg: 'Approval error readable' },
				value: undefined
			});
			vi.mocked(mockVault.getDepositCalldata).mockResolvedValue({
				error: { msg: 'Deposit error', readableMsg: 'Deposit error readable' },
				value: undefined
			});

			const onSubmitCall = mockHandleDepositModal.mock.calls[0][0].onSubmit;
			await onSubmitCall(mockAmount);

			expect(mockErrToast).toHaveBeenCalledWith('Deposit error');
			// ensure no transaction confirmation modal for deposit is shown
			expect(mockHandleTransactionConfirmationModal).not.toHaveBeenCalled();
		});

		it('should handle approval and then deposit if approvalCalldata is successful', async () => {
			vi.mocked(mockVault.getApprovalCalldata).mockResolvedValue({
				value: mockApprovalCalldata,
				error: undefined
			});
			vi.mocked(mockVault.getDepositCalldata).mockResolvedValue({
				value: mockDepositCalldata,
				error: undefined
			});

			const onSubmitCall = mockHandleDepositModal.mock.calls[0][0].onSubmit;
			await onSubmitCall(mockAmount);

			expect(mockVault.getApprovalCalldata).toHaveBeenCalledTimes(1);
			expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(1);
			expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(1, {
				open: true,
				modalTitle: 'Approving TEST spend',
				closeOnConfirm: true,
				args: {
					entity: mockVault,
					toAddress: mockVault.token.address as Hex,
					chainId: mockVault.chainId,
					onConfirm: expect.any(Function),
					calldata: mockApprovalCalldata
				}
			});

			// Simulate approval confirmation
			const onApprovalConfirmCall =
				mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
			onApprovalConfirmCall(mockTxHashApproval);

			expect(mockCreateApprovalTransaction).toHaveBeenCalledWith({
				txHash: mockTxHashApproval,
				chainId: mockVault.chainId,
				queryKey: mockVault.id,
				entity: mockVault
			});

			expect(mockVault.getDepositCalldata).toHaveBeenCalledWith(mockAmount);
			await waitFor(() => {
				expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(2);
				expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(2, {
					open: true,
					modalTitle: 'Depositing 100 TEST',
					closeOnConfirm: false,
					args: {
						entity: mockVault,
						toAddress: mockVault.raindex,
						chainId: mockVault.chainId,
						onConfirm: expect.any(Function),
						calldata: mockDepositCalldata
					}
				});
			});

			const onDepositConfirmCall =
				mockHandleTransactionConfirmationModal.mock.calls[1][0].args.onConfirm;
			onDepositConfirmCall(mockTxHashDeposit);

			expect(mockCreateDepositTransaction).toHaveBeenCalledWith({
				raindexClient: mockRaindexClient,
				txHash: mockTxHashDeposit,
				chainId: mockVault.chainId,
				queryKey: mockVault.id,
				entity: mockVault,
				amount: mockAmount
			});
		});

		it('should show error toast if getVaultDepositCalldata fails after successful approval', async () => {
			vi.mocked(mockVault.getApprovalCalldata).mockResolvedValue({
				value: mockApprovalCalldata,
				error: undefined
			});
			vi.mocked(mockVault.getDepositCalldata).mockResolvedValue({
				error: { msg: 'Deposit error after approval', readableMsg: 'Deposit error readable' },
				value: undefined
			});

			const onSubmitCall = mockHandleDepositModal.mock.calls[0][0].onSubmit;
			await onSubmitCall(mockAmount);

			const onApprovalConfirmCall =
				mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
			await onApprovalConfirmCall(mockTxHashApproval);

			expect(mockErrToast).toHaveBeenCalledWith('Deposit error after approval');
			expect(
				mockHandleTransactionConfirmationModal.mock.calls.filter(
					(call) => call[0].args.calldata === mockDepositCalldata
				).length
			).toBe(0);
		});
	});
});
