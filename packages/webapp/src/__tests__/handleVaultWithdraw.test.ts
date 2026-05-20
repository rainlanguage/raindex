import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
	handleVaultWithdraw,
	type VaultWithdrawHandlerDependencies
} from '../lib/services/handleVaultWithdraw';
import { Float, type RaindexClient, type RaindexVault } from '@rainlanguage/raindex';
import type { Hex } from 'viem';
import type { TransactionManager } from '@rainlanguage/ui-components';

// Mocks
const mockHandleWithdrawModal = vi.fn();
const mockHandleTransactionConfirmationModal = vi.fn();
const mockErrToast = vi.fn();
const mockCreateWithdrawTransaction = vi.fn();

const mockManager = {
	createWithdrawTransaction: mockCreateWithdrawTransaction
};

const mockRaindexClient = {} as unknown as RaindexClient;

const mockVault = {
	id: '0xvaultid',
	token: {
		symbol: 'TEST'
	},
	getWithdrawCalldata: vi.fn()
} as unknown as RaindexVault;

const mockDeps: VaultWithdrawHandlerDependencies = {
	raindexClient: mockRaindexClient,
	vault: mockVault,
	account: '0xaccount' as Hex,
	handleWithdrawModal: mockHandleWithdrawModal,
	handleTransactionConfirmationModal: mockHandleTransactionConfirmationModal,
	errToast: mockErrToast,
	manager: mockManager as unknown as TransactionManager
};

describe('handleVaultWithdraw', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should call handleWithdrawModal with correct arguments', async () => {
		await handleVaultWithdraw(mockDeps);
		expect(mockHandleWithdrawModal).toHaveBeenCalledWith({
			open: true,
			args: {
				vault: mockVault,
				account: mockDeps.account
			},
			onSubmit: expect.any(Function)
		});
	});

	it('should show error toast if getVaultWithdrawCalldata returns an error', async () => {
		vi.mocked(mockVault.getWithdrawCalldata).mockResolvedValue({
			error: { msg: 'Calldata error', readableMsg: 'Calldata error readable' },
			value: undefined
		});

		await handleVaultWithdraw(mockDeps);

		const onSubmitCall = mockHandleWithdrawModal.mock.calls[0][0].onSubmit;
		await onSubmitCall(Float.parse('100').value as Float);

		expect(mockErrToast).toHaveBeenCalledWith('Calldata error');
		expect(mockHandleTransactionConfirmationModal).not.toHaveBeenCalled();
	});

	it('should show error toast if getVaultWithdrawCalldata throws', async () => {
		vi.mocked(mockVault.getWithdrawCalldata).mockRejectedValue(new Error('Fetch failed'));

		await handleVaultWithdraw(mockDeps);

		const onSubmitCall = mockHandleWithdrawModal.mock.calls[0][0].onSubmit;
		await onSubmitCall(Float.parse('100').value as Float);

		expect(mockErrToast).toHaveBeenCalledWith('Failed to get calldata for vault withdrawal.');
		expect(mockHandleTransactionConfirmationModal).not.toHaveBeenCalled();
	});

	it('should call handleTransactionConfirmationModal on successful calldata fetch', async () => {
		const mockCalldata = '0xcalldata' as Hex;
		vi.mocked(mockVault.getWithdrawCalldata).mockResolvedValue({
			value: mockCalldata,
			error: undefined
		});

		await handleVaultWithdraw(mockDeps);

		const onSubmitCall = mockHandleWithdrawModal.mock.calls[0][0].onSubmit;
		await onSubmitCall(Float.parse('0.1').value as Float);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledWith({
			open: true,
			modalTitle: 'Withdrawing 0.1 TEST...',
			args: {
				entity: mockVault,
				onConfirm: expect.any(Function),
				calldata: mockCalldata
			}
		});
		expect(mockErrToast).not.toHaveBeenCalled();
	});

	it('should call manager.createWithdrawTransaction on transaction confirmation', async () => {
		const mockCalldata = '0xcalldata' as Hex;
		const mockTxHash = '0xtxhash' as Hex;
		vi.mocked(mockVault.getWithdrawCalldata).mockResolvedValue({
			value: mockCalldata,
			error: undefined
		});

		await handleVaultWithdraw(mockDeps);

		const onSubmitCall = mockHandleWithdrawModal.mock.calls[0][0].onSubmit;
		await onSubmitCall(Float.parse('100').value as Float);

		const onConfirmCall = mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
		onConfirmCall(mockTxHash);

		expect(mockCreateWithdrawTransaction).toHaveBeenCalledWith({
			raindexClient: mockRaindexClient,
			entity: mockVault,
			txHash: mockTxHash,
			chainId: mockVault.chainId,
			queryKey: mockVault.id
		});
	});
});
