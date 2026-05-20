import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
	handleRemoveOrder,
	type HandleRemoveOrderDependencies
} from '../lib/services/handleRemoveOrder'; // Assuming path is correct
import type { RaindexClient, RaindexOrder } from '@rainlanguage/raindex';
import type { Hex } from 'viem';
import type { TransactionManager } from '@rainlanguage/ui-components';

// Mocks
const mockHandleTransactionConfirmationModal = vi.fn();
const mockErrToast = vi.fn();
const mockCreateRemoveOrderTransaction = vi.fn();

const mockManager = {
	createRemoveOrderTransaction: mockCreateRemoveOrderTransaction
};

const mockRaindexClient = {} as unknown as RaindexClient;

const mockOrder = {
	id: '0xorderid',
	orderHash: '0xorderhashfromparams',
	getRemoveCalldata: vi.fn()
} as unknown as RaindexOrder;

const mockDeps: HandleRemoveOrderDependencies = {
	raindexClient: mockRaindexClient,
	order: mockOrder,
	handleTransactionConfirmationModal: mockHandleTransactionConfirmationModal,
	errToast: mockErrToast,
	manager: mockManager as unknown as TransactionManager
};

describe('handleRemoveOrder', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should show error toast if getRemoveCalldata returns an error', async () => {
		vi.mocked(mockOrder.getRemoveCalldata).mockReturnValue({
			error: { msg: 'Calldata error', readableMsg: 'Calldata error readable' },
			value: undefined
		});

		await handleRemoveOrder(mockDeps);

		expect(mockErrToast).toHaveBeenCalledWith('Calldata error readable');
		expect(mockHandleTransactionConfirmationModal).not.toHaveBeenCalled();
	});

	it('should show error toast if getRemoveOrderCalldata throws', async () => {
		vi.mocked(mockOrder.getRemoveCalldata).mockImplementation(() => {
			throw new Error('Fetch failed');
		});

		await handleRemoveOrder(mockDeps);

		expect(mockErrToast).toHaveBeenCalledWith('Failed to get calldata for order removal.');
		expect(mockHandleTransactionConfirmationModal).not.toHaveBeenCalled();
	});

	it('should call handleTransactionConfirmationModal on successful calldata fetch', async () => {
		const mockCalldata = '0xcalldata' as Hex;
		vi.mocked(mockOrder.getRemoveCalldata).mockReturnValue({
			value: mockCalldata,
			error: undefined
		});

		await handleRemoveOrder(mockDeps);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledWith({
			open: true,
			modalTitle: 'Removing order',
			args: {
				entity: mockOrder,
				toAddress: mockOrder.raindex,
				chainId: mockOrder.chainId,
				onConfirm: expect.any(Function),
				calldata: mockCalldata
			}
		});
		expect(mockErrToast).not.toHaveBeenCalled();
	});

	it('should call manager.createRemoveOrderTransaction on transaction confirmation', async () => {
		const mockCalldata = '0xcalldata' as Hex;
		const mockTxHash = '0xtxhash' as Hex;
		vi.mocked(mockOrder.getRemoveCalldata).mockReturnValue({
			value: mockCalldata,
			error: undefined
		});

		await handleRemoveOrder(mockDeps);

		const onConfirmCall = mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
		onConfirmCall(mockTxHash);

		expect(mockCreateRemoveOrderTransaction).toHaveBeenCalledWith({
			raindexClient: mockRaindexClient,
			txHash: mockTxHash,
			queryKey: mockOrder.orderHash,
			chainId: mockOrder.chainId,
			entity: mockOrder
		});
	});

	it('should call handleTransactionConfirmationModal with correct modalTitle when removing an order', async () => {
		const mockCalldata = '0xmockcalldata';
		vi.mocked(mockOrder.getRemoveCalldata).mockReturnValue({
			value: mockCalldata,
			error: undefined
		});

		await handleRemoveOrder(mockDeps);

		expect(mockDeps.handleTransactionConfirmationModal).toHaveBeenCalledOnce();
		expect(mockDeps.handleTransactionConfirmationModal).toHaveBeenCalledWith(
			expect.objectContaining({
				modalTitle: 'Removing order'
			})
		);
	});
});
