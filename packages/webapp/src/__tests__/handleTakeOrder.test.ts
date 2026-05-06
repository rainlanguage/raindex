import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
	handleTakeOrder,
	type TakeOrderHandlerDependencies
} from '../lib/services/handleTakeOrder';
import type { TakeOrderSubmitParams } from '../lib/services/modal';
import {
	Float,
	type RaindexClient,
	type RaindexOrder,
	type TakeOrdersCalldataResult
} from '@rainlanguage/orderbook';
import type { Hex } from 'viem';
import type { TransactionManager } from '@rainlanguage/ui-components';

const mockHandleTakeOrderModal = vi.fn();
const mockHandleTransactionConfirmationModal = vi.fn().mockResolvedValue({ success: true });
const mockErrToast = vi.fn();
const mockCreateTakeOrderTransaction = vi.fn();
const mockCreateApprovalTransaction = vi.fn();

const mockManager = {
	createTakeOrderTransaction: mockCreateTakeOrderTransaction,
	createApprovalTransaction: mockCreateApprovalTransaction
};

const mockRaindexClient = {} as unknown as RaindexClient;

const MOCK_ORDERBOOK_ADDRESS = '0x1234567890123456789012345678901234567890' as Hex;
const MOCK_TOKEN_ADDRESS = '0xAbcDef1234567890AbcDef1234567890AbcDef12' as Hex;
const MOCK_ACCOUNT_ADDRESS = '0x9876543210987654321098765432109876543210' as Hex;

const mockOrder = {
	id: '0xorderid',
	orderHash: '0xorderhash',
	chainId: 1,
	orderbook: MOCK_ORDERBOOK_ADDRESS,
	inputsList: {
		items: [
			{
				token: {
					address: MOCK_TOKEN_ADDRESS,
					decimals: 18,
					symbol: 'TEST'
				}
			}
		]
	},
	getTakeCalldata: vi.fn()
} as unknown as RaindexOrder;

const mockQuote = {
	pair: {
		pairName: 'USDC/TEST',
		inputIndex: 0,
		outputIndex: 0
	},
	success: true,
	data: {
		maxOutput: Float.parse('100').value,
		ratio: Float.parse('1.5').value
	}
};

const mockDeps: TakeOrderHandlerDependencies = {
	raindexClient: mockRaindexClient,
	order: mockOrder,
	handleTakeOrderModal: mockHandleTakeOrderModal,
	handleTransactionConfirmationModal: mockHandleTransactionConfirmationModal,
	errToast: mockErrToast,
	manager: mockManager as unknown as TransactionManager,
	account: MOCK_ACCOUNT_ADDRESS
};

const mockParams: TakeOrderSubmitParams = {
	quote: mockQuote as never,
	mode: 'buyUpTo' as never,
	amount: '10',
	priceCap: '2.0'
};

function createMockReadyResult(calldata: string, orderbook: string): TakeOrdersCalldataResult {
	return {
		isReady: true,
		isNeedsApproval: false,
		approvalInfo: undefined,
		takeOrdersInfo: {
			calldata,
			orderbook,
			effectivePrice: Float.parse('1.5').value as Float,
			maxSellCap: Float.parse('15').value as Float,
			expectedSell: Float.parse('15').value as Float,
			prices: []
		}
	} as unknown as TakeOrdersCalldataResult;
}

function createMockApprovalResult(
	token: string,
	approvalCalldata: string
): TakeOrdersCalldataResult {
	return {
		isReady: false,
		isNeedsApproval: true,
		approvalInfo: {
			token,
			calldata: approvalCalldata,
			spender: '0xspender',
			amount: Float.parse('100').value as Float,
			formattedAmount: '100'
		},
		takeOrdersInfo: undefined
	} as unknown as TakeOrdersCalldataResult;
}

async function flushPromises() {
	await new Promise((resolve) => setTimeout(resolve, 0));
}

async function triggerOnSubmit(params: TakeOrderSubmitParams = mockParams) {
	const onSubmit = mockHandleTakeOrderModal.mock.calls[0][0].onSubmit;
	onSubmit(params);
	await flushPromises();
}

describe('handleTakeOrder', () => {
	beforeEach(() => {
		vi.resetAllMocks();
	});

	it('should open the take order modal with correct props', async () => {
		await handleTakeOrder(mockDeps);

		expect(mockHandleTakeOrderModal).toHaveBeenCalledTimes(1);
		expect(mockHandleTakeOrderModal).toHaveBeenCalledWith({
			open: true,
			order: mockOrder,
			onSubmit: expect.any(Function)
		});
	});

	it('should show error toast if input token cannot be determined (invalid inputIndex)', async () => {
		const invalidQuote = {
			...mockQuote,
			pair: {
				...mockQuote.pair,
				inputIndex: 99
			}
		};

		await handleTakeOrder(mockDeps);
		await triggerOnSubmit({ ...mockParams, quote: invalidQuote as never });

		expect(mockErrToast).toHaveBeenCalledWith('Could not determine input token for this order');
		expect(mockHandleTransactionConfirmationModal).not.toHaveBeenCalled();
	});

	it('should show error toast if getTakeCalldata returns an error', async () => {
		vi.mocked(mockOrder.getTakeCalldata).mockResolvedValue({
			error: { msg: 'Calldata error', readableMsg: 'No liquidity available' },
			value: undefined
		});

		await handleTakeOrder(mockDeps);
		await triggerOnSubmit();

		expect(mockErrToast).toHaveBeenCalledWith('No liquidity available');
		expect(mockHandleTransactionConfirmationModal).not.toHaveBeenCalled();
	});

	it('should execute take order with correct calldata', async () => {
		const mockCalldata = '0xcalldata' as Hex;

		vi.mocked(mockOrder.getTakeCalldata).mockResolvedValue({
			value: createMockReadyResult(mockCalldata, MOCK_ORDERBOOK_ADDRESS),
			error: undefined
		});

		await handleTakeOrder(mockDeps);
		await triggerOnSubmit();

		expect(mockErrToast).not.toHaveBeenCalled();

		expect(mockOrder.getTakeCalldata).toHaveBeenCalledWith(
			mockParams.quote.pair.inputIndex,
			mockParams.quote.pair.outputIndex,
			mockDeps.account,
			mockParams.mode,
			mockParams.amount,
			mockParams.priceCap
		);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(1);
		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledWith({
			open: true,
			modalTitle: 'Taking order for TEST',
			closeOnConfirm: false,
			args: expect.objectContaining({
				toAddress: MOCK_ORDERBOOK_ADDRESS,
				chainId: mockOrder.chainId,
				calldata: mockCalldata
			})
		});
	});

	it('should call manager.createTakeOrderTransaction on transaction confirmation', async () => {
		const mockCalldata = '0xcalldata' as Hex;
		const mockTxHash = '0xtxhash' as Hex;

		vi.mocked(mockOrder.getTakeCalldata).mockResolvedValue({
			value: createMockReadyResult(mockCalldata, MOCK_ORDERBOOK_ADDRESS),
			error: undefined
		});

		await handleTakeOrder(mockDeps);
		await triggerOnSubmit();

		const onConfirmCall = mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
		onConfirmCall(mockTxHash);

		expect(mockCreateTakeOrderTransaction).toHaveBeenCalledWith({
			raindexClient: mockRaindexClient,
			txHash: mockTxHash,
			chainId: mockOrder.chainId,
			queryKey: mockOrder.orderHash,
			entity: mockOrder
		});
	});

	it('should handle empty inputsList gracefully', async () => {
		const orderWithNoInputs = {
			...mockOrder,
			inputsList: { items: [] }
		} as unknown as RaindexOrder;

		await handleTakeOrder({ ...mockDeps, order: orderWithNoInputs });
		await triggerOnSubmit();

		expect(mockErrToast).toHaveBeenCalledWith('Could not determine input token for this order');
	});

	it('should handle null inputsList gracefully', async () => {
		const orderWithNullInputs = {
			...mockOrder,
			inputsList: null
		} as unknown as RaindexOrder;

		await handleTakeOrder({ ...mockDeps, order: orderWithNullInputs });
		await triggerOnSubmit();

		expect(mockErrToast).toHaveBeenCalledWith('Could not determine input token for this order');
	});

	it('should use token symbol from order when available', async () => {
		const mockCalldata = '0xcalldata' as Hex;

		const orderWithCustomSymbol = {
			...mockOrder,
			inputsList: {
				items: [
					{
						token: {
							address: MOCK_TOKEN_ADDRESS,
							decimals: 18,
							symbol: 'CUSTOM'
						}
					}
				]
			}
		} as unknown as RaindexOrder;

		vi.mocked(orderWithCustomSymbol.getTakeCalldata).mockResolvedValue({
			value: createMockReadyResult(mockCalldata, MOCK_ORDERBOOK_ADDRESS),
			error: undefined
		});

		await handleTakeOrder({ ...mockDeps, order: orderWithCustomSymbol });
		await triggerOnSubmit();

		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledWith(
			expect.objectContaining({
				modalTitle: 'Taking order for CUSTOM'
			})
		);
	});

	it('should default to "tokens" when token symbol is missing', async () => {
		const mockCalldata = '0xcalldata' as Hex;

		const orderWithNoSymbol = {
			...mockOrder,
			inputsList: {
				items: [
					{
						token: {
							address: MOCK_TOKEN_ADDRESS,
							decimals: 18,
							symbol: null
						}
					}
				]
			},
			getTakeCalldata: vi.fn()
		} as unknown as RaindexOrder;

		vi.mocked(orderWithNoSymbol.getTakeCalldata).mockResolvedValue({
			value: createMockReadyResult(mockCalldata, MOCK_ORDERBOOK_ADDRESS),
			error: undefined
		});

		await handleTakeOrder({ ...mockDeps, order: orderWithNoSymbol });
		await triggerOnSubmit();

		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledWith(
			expect.objectContaining({
				modalTitle: 'Taking order for tokens'
			})
		);
	});

	it('should show approval modal when isNeedsApproval is true', async () => {
		const mockApprovalCalldata = '0xapprovalcalldata' as Hex;

		mockHandleTransactionConfirmationModal.mockResolvedValue({ success: true });

		vi.mocked(mockOrder.getTakeCalldata).mockResolvedValue({
			value: createMockApprovalResult(MOCK_TOKEN_ADDRESS, mockApprovalCalldata),
			error: undefined
		});

		await handleTakeOrder(mockDeps);
		await triggerOnSubmit();

		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(1);
		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledWith({
			open: true,
			modalTitle: 'Approving TEST spend',
			closeOnConfirm: true,
			args: expect.objectContaining({
				toAddress: MOCK_TOKEN_ADDRESS,
				chainId: mockOrder.chainId,
				calldata: mockApprovalCalldata
			})
		});
	});

	it('should automatically execute take order after approval confirmation', async () => {
		const mockApprovalCalldata = '0xapprovalcalldata' as Hex;
		const mockTakeCalldata = '0xtakecalldata' as Hex;
		const mockTxHash = '0xapprovaltxhash' as Hex;

		vi.mocked(mockOrder.getTakeCalldata)
			.mockResolvedValueOnce({
				value: createMockApprovalResult(MOCK_TOKEN_ADDRESS, mockApprovalCalldata),
				error: undefined
			})
			.mockResolvedValueOnce({
				value: createMockReadyResult(mockTakeCalldata, MOCK_ORDERBOOK_ADDRESS),
				error: undefined
			});

		await handleTakeOrder(mockDeps);
		await triggerOnSubmit();

		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(1);

		const onConfirmCall = mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
		await onConfirmCall(mockTxHash);
		await flushPromises();

		expect(mockOrder.getTakeCalldata).toHaveBeenCalledTimes(2);
		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(2);
		expect(mockHandleTransactionConfirmationModal).toHaveBeenLastCalledWith(
			expect.objectContaining({
				modalTitle: 'Taking order for TEST',
				closeOnConfirm: false
			})
		);
	});

	it('should call manager.createApprovalTransaction on approval confirmation', async () => {
		const mockApprovalCalldata = '0xapprovalcalldata' as Hex;
		const mockTxHash = '0xapprovaltxhash' as Hex;

		mockHandleTransactionConfirmationModal.mockResolvedValue({ success: true });

		vi.mocked(mockOrder.getTakeCalldata).mockResolvedValue({
			value: createMockApprovalResult(MOCK_TOKEN_ADDRESS, mockApprovalCalldata),
			error: undefined
		});

		await handleTakeOrder(mockDeps);
		await triggerOnSubmit();

		const onConfirmCall = mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
		onConfirmCall(mockTxHash);

		expect(mockCreateApprovalTransaction).toHaveBeenCalledWith({
			txHash: mockTxHash,
			chainId: mockOrder.chainId,
			queryKey: mockOrder.orderHash
		});
	});
});
