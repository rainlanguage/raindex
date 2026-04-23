import {
	type RaindexClient,
	type RaindexOrder,
	type RaindexOrderQuote,
	type TakeOrdersInfo
} from '@rainlanguage/orderbook';
import { type Hex } from 'viem';
import type {
	TransactionManager,
	HandleTransactionConfirmationModal
} from '@rainlanguage/ui-components';
import type { HandleTakeOrderModal, TakeOrderSubmitParams } from './modal';

export interface TakeOrderHandlerDependencies {
	raindexClient: RaindexClient;
	order: RaindexOrder;
	handleTakeOrderModal: HandleTakeOrderModal;
	handleTransactionConfirmationModal: HandleTransactionConfirmationModal;
	errToast: (message: string) => void;
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

async function executeTakeOrderWithFreshCalldata(
	deps: Omit<TakeOrderHandlerDependencies, 'handleTakeOrderModal'>,
	params: TakeOrderSubmitParams,
	inputTokenSymbol: string
): Promise<void> {
	const { order, errToast, account } = deps;
	const { quote, mode, amount, priceCap } = params;

	const calldataResult = await order.getTakeCalldata(
		quote.pair.inputIndex,
		quote.pair.outputIndex,
		account,
		mode,
		amount,
		priceCap
	);
	if (calldataResult.error) {
		errToast(calldataResult.error.readableMsg);
		return;
	}

	const result = calldataResult.value;
	if (!result.isReady || !result.takeOrdersInfo) {
		errToast('Failed to get take order calldata after approval');
		return;
	}

	await executeTakeOrder({
		...deps,
		takeOrdersInfo: result.takeOrdersInfo,
		inputTokenSymbol
	});
}

async function processSubmit(
	deps: TakeOrderHandlerDependencies,
	params: TakeOrderSubmitParams
): Promise<void> {
	const { order, errToast, account, handleTransactionConfirmationModal, manager } = deps;
	const { quote, mode, amount, priceCap } = params;

	try {
		const inputTokenSymbol = getInputTokenSymbol(order, quote);
		if (!inputTokenSymbol) {
			errToast('Could not determine input token for this order');
			return;
		}

		const calldataResult = await order.getTakeCalldata(
			quote.pair.inputIndex,
			quote.pair.outputIndex,
			account,
			mode,
			amount,
			priceCap
		);
		if (calldataResult.error) {
			errToast(calldataResult.error.readableMsg);
			return;
		}

		const result = calldataResult.value;

		if (result.isNeedsApproval) {
			const approvalInfo = result.approvalInfo!;

			handleTransactionConfirmationModal({
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
						executeTakeOrderWithFreshCalldata(deps, params, inputTokenSymbol);
					}
				}
			});
			return;
		}

		if (!result.isReady) {
			errToast('Unexpected state from take order calldata');
			return;
		}

		const takeOrdersInfo = result.takeOrdersInfo!;
		await executeTakeOrder({
			...deps,
			takeOrdersInfo,
			inputTokenSymbol
		});
	} catch {
		errToast('Failed to get calldata for take order.');
	}
}

export async function handleTakeOrder(deps: TakeOrderHandlerDependencies): Promise<void> {
	const { order, handleTakeOrderModal } = deps;

	handleTakeOrderModal({
		open: true,
		order,
		onSubmit: async (params: TakeOrderSubmitParams) => {
			await processSubmit(deps, params);
		}
	});
}
