import type { RaindexClient, RaindexOrder } from '@rainlanguage/raindex';
import type { Hex } from 'viem';
import type { TransactionManager, TransactionConfirmationProps } from '@rainlanguage/ui-components';

export interface HandleRemoveOrderDependencies {
	raindexClient: RaindexClient;
	order: RaindexOrder;
	handleTransactionConfirmationModal: (props: TransactionConfirmationProps) => void;
	errToast: (message: string) => void;
	manager: TransactionManager;
}

export async function handleRemoveOrder(deps: HandleRemoveOrderDependencies): Promise<void> {
	let calldata: string;
	try {
		const calldataResult = deps.order.getRemoveCalldata();
		if (calldataResult.error) {
			return deps.errToast(calldataResult.error.readableMsg);
		}
		calldata = calldataResult.value;
		deps.handleTransactionConfirmationModal({
			open: true,
			modalTitle: 'Removing order',
			args: {
				entity: deps.order,
				toAddress: deps.order.raindex,
				chainId: deps.order.chainId,
				onConfirm: (txHash: Hex) => {
					deps.manager.createRemoveOrderTransaction({
						raindexClient: deps.raindexClient,
						txHash,
						queryKey: deps.order.orderHash,
						chainId: deps.order.chainId,
						entity: deps.order
					});
				},
				calldata
			}
		});
	} catch {
		deps.errToast('Failed to get calldata for order removal.');
	}
}
