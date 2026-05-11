import type {
	Address,
	RaindexOrderBuilder,
	RaindexClient,
	RaindexVault,
	RaindexVaultToken
} from '@rainlanguage/orderbook';
import type {
	TransactionManager,
	HandleTransactionConfirmationModal
} from '@rainlanguage/ui-components';
import { QKEY_ORDERS } from '@rainlanguage/ui-components';
import { type Hex } from 'viem';

export enum AddOrderErrors {
	ADD_ORDER_FAILED = 'Failed to add order',
	MISSING_BUILDER = 'Order builder is required',
	MISSING_CONFIG = 'Wagmi config is required',
	NO_ACCOUNT_CONNECTED = 'No wallet address found',
	ERROR_GETTING_ARGS = 'Error getting deployment transaction args',
	ERROR_GETTING_NETWORK_KEY = 'Error getting network key'
}

export type HandleAddOrderDependencies = {
	handleTransactionConfirmationModal: HandleTransactionConfirmationModal;
	errToast: (message: string) => void;
	manager: TransactionManager;
	builder: RaindexOrderBuilder;
	raindexClient: RaindexClient;
	account: Hex | null;
};

export const handleAddOrder = async (deps: HandleAddOrderDependencies) => {
	const { builder, account, errToast, raindexClient } = deps;

	if (!account) {
		return errToast('Could not deploy: ' + AddOrderErrors.NO_ACCOUNT_CONNECTED);
	}

	const result = await builder.getDeploymentTransactionArgs(account);

	if (result.error) {
		return errToast('Could not deploy: ' + result.error.msg);
	}

	const { approvals, deploymentCalldata, orderbookAddress, chainId, emitMetaCall } = result.value;

	for (const approval of approvals) {
		try {
			const approvalResult = await deps.handleTransactionConfirmationModal({
				open: true,
				modalTitle: `Approving ${approval.symbol} spend`,
				closeOnConfirm: true,
				args: {
					toAddress: approval.token as Hex,
					chainId,
					calldata: approval.calldata as `0x${string}`,
					onConfirm: (hash: Hex) => {
						deps.manager.createApprovalTransaction({
							chainId,
							txHash: hash,
							queryKey: QKEY_ORDERS,
							entity: {
								token: { symbol: approval.symbol } as unknown as RaindexVaultToken
							} as unknown as RaindexVault
						});
					}
				}
			});

			if (!approvalResult.success) {
				return errToast('Approval was cancelled or failed');
			}
		} catch (error) {
			return errToast('Approval failed: ' + (error as Error).message);
		}
	}

	if (emitMetaCall) {
		try {
			const metaResult = await deps.handleTransactionConfirmationModal({
				open: true,
				modalTitle: 'Publishing metadata',
				args: {
					toAddress: emitMetaCall.to as Hex,
					chainId,
					calldata: emitMetaCall.calldata as `0x${string}`,
					onConfirm: (hash: Hex) => {
						deps.manager.createMetaTransaction({
							chainId,
							txHash: hash,
							queryKey: QKEY_ORDERS
						});
					}
				}
			});

			if (!metaResult.success) {
				return errToast('Metadata publication was cancelled or failed');
			}
		} catch (error) {
			return errToast('Metadata publication failed: ' + (error as Error).message);
		}
	}

	try {
		const deploymentResult = await deps.handleTransactionConfirmationModal({
			open: true,
			modalTitle: 'Deploying your order',
			args: {
				toAddress: orderbookAddress as Address,
				chainId,
				calldata: deploymentCalldata as `0x${string}`,
				onConfirm: (hash: Hex) => {
					deps.manager.createAddOrderTransaction({
						raindexClient,
						orderbook: orderbookAddress as Address,
						chainId,
						txHash: hash,
						queryKey: QKEY_ORDERS
					});
				}
			}
		});

		if (!deploymentResult.success) {
			return errToast('Deployment was cancelled or failed');
		}
	} catch (error) {
		return errToast('Deployment failed: ' + (error as Error).message);
	}
};
