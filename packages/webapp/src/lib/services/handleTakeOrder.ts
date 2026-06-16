import {
	type RaindexClient,
	type RaindexOrder,
	type RaindexOrderQuote,
	type TakeOrdersInfo
} from '@rainlanguage/orderbook';
import { type Hex } from 'viem';
import type {
	TransactionManager,
	HandleTransactionConfirmationModal,
	ToastProps
} from '@rainlanguage/ui-components';
import type { HandleTakeOrderModal, TakeOrderSubmitParams } from './modal';

export interface TakeOrderHandlerDependencies {
	raindexClient: RaindexClient;
	order: RaindexOrder;
	handleTakeOrderModal: HandleTakeOrderModal;
	handleTransactionConfirmationModal: HandleTransactionConfirmationModal;
	errToast: (message: string) => void;
	addToast: (toast: ToastProps) => void;
	manager: TransactionManager;
	account: Hex;
}

export type ExecuteTakeOrderArgs = Omit<TakeOrderHandlerDependencies, 'handleTakeOrderModal'> & {
	takeOrdersInfo: TakeOrdersInfo;
	inputTokenSymbol: string;
};

function getInputTokenSymbol(order: RaindexOrder, quote: RaindexOrderQuote): string | null {
	const inputIndex = quote.pair.inputIndex;
	const inputs = order.inputsList?.items ?? [];
	if (inputIndex >= inputs.length) {
		return null;
	}
	return inputs[inputIndex].token.symbol ?? 'tokens';
}

async function executeTakeOrder(args: ExecuteTakeOrderArgs): Promise<void> {
	const {
		raindexClient,
		order,
		takeOrdersInfo,
		handleTransactionConfirmationModal,
		manager,
		inputTokenSymbol
	} = args;

	handleTransactionConfirmationModal({
		open: true,
		modalTitle: `Taking order for ${inputTokenSymbol}`,
		closeOnConfirm: false,
		args: {
			entity: order,
			toAddress: takeOrdersInfo.orderbook as Hex,
			chainId: order.chainId,
			onConfirm: (txHash: Hex) => {
				manager.createTakeOrderTransaction({
					raindexClient,
					txHash,
					chainId: order.chainId,
					queryKey: order.orderHash,
					entity: order
				});
			},
			calldata: takeOrdersInfo.calldata as string
		}
	});
}

async function processSubmit(
	deps: TakeOrderHandlerDependencies,
	params: TakeOrderSubmitParams
): Promise<boolean> {
	const { order, errToast, addToast, account, handleTransactionConfirmationModal, manager } = deps;
	const { quote, mode, amount, priceCap } = params;

	try {
		const inputTokenSymbol = getInputTokenSymbol(order, quote);
		if (!inputTokenSymbol) {
			errToast('Could not determine input token for this order');
			return true;
		}

		console.log('DEBUG: Take order params:', {
			inputIndex: quote.pair.inputIndex,
			outputIndex: quote.pair.outputIndex,
			account,
			mode,
			amount: amount.toString(),
			priceCap: priceCap.toString(),
			maxOutput: quote.data?.maxOutput.toString(),
			ratio: quote.data?.ratio.toString()
		});

		const calldataResult = await order.getTakeCalldata(
			quote.pair.inputIndex,
			quote.pair.outputIndex,
			account,
			mode,
			amount,
			priceCap
		);
		
		console.log('DEBUG: Calldata result:', {
			error: calldataResult.error?.readableMsg,
			isNeedsApproval: calldataResult.value?.isNeedsApproval,
			isReady: calldataResult.value?.isReady
		});
		
		if (calldataResult.error) {
			console.log('DEBUG: Take order failed with error:', calldataResult.error);
			errToast(calldataResult.error.readableMsg);
			return true;
		}

		const result = calldataResult.value;

		if (result.isNeedsApproval) {
			const approvalInfo = result.approvalInfo!;

			const approvalResult = await handleTransactionConfirmationModal({
				open: true,
				modalTitle: `Approving ${inputTokenSymbol} spend`,
				closeOnConfirm: true,
				args: {
					toAddress: approvalInfo.token as Hex,
					chainId: order.chainId,
					calldata: approvalInfo.calldata as string,
					onConfirm: (hash: Hex) => {
						manager.createApprovalTransaction({
							txHash: hash,
							chainId: order.chainId,
							queryKey: order.orderHash
						});
					}
				}
			});

			if (!approvalResult.success) {
				errToast('Approval was cancelled or failed');
				return true;
			}

			addToast({
				message: "Approval successful! Click 'Take Order' again to proceed with fresh quotes.",
				type: 'success',
				color: 'green'
			});
			return false;
		}

		if (!result.isReady) {
			errToast('Unexpected state from take order calldata');
			return true;
		}

		const takeOrdersInfo = result.takeOrdersInfo!;
		await executeTakeOrder({
			...deps,
			takeOrdersInfo,
			inputTokenSymbol
		});
		return true;
	} catch {
		errToast('Failed to get calldata for take order.');
		return true;
	}
}

export async function handleTakeOrder(deps: TakeOrderHandlerDependencies): Promise<void> {
	const { order, handleTakeOrderModal } = deps;

	handleTakeOrderModal({
		open: true,
		order,
		onSubmit: async (params: TakeOrderSubmitParams) => {
			return await processSubmit(deps, params);
		}
	});
}
