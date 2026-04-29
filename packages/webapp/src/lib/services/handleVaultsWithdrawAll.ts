import type { RaindexClient, RaindexVault, RaindexVaultsList } from '@rainlanguage/raindex';
import { type Hex } from 'viem';
import { QKEY_VAULTS, type TransactionManager } from '@rainlanguage/ui-components';
import type { TransactionConfirmationProps } from '@rainlanguage/ui-components';
import type { TransactionConfirmationModalResult } from './modal';

export type WithdrawAllModalProps = {
	open: boolean;
	vaults: RaindexVault[];
	onSubmit: () => void;
};

export interface MultipleVaultsWithdrawHandlerDependencies {
	raindexClient: RaindexClient;
	vaultsList: RaindexVaultsList;
	handleWithdrawAllModal: (props: WithdrawAllModalProps) => void;
	handleTransactionConfirmationModal: (
		props: TransactionConfirmationProps
	) => Promise<TransactionConfirmationModalResult>;
	errToast: (message: string) => void;
	manager: TransactionManager;
	account: Hex;
}

export async function handleVaultsWithdrawAll(
	deps: MultipleVaultsWithdrawHandlerDependencies
): Promise<boolean> {
	const {
		raindexClient,
		vaultsList,
		handleWithdrawAllModal,
		handleTransactionConfirmationModal,
		errToast,
		manager
	} = deps;
	const vaultsResult = vaultsList.getWithdrawableVaults();
	if (vaultsResult.error) {
		errToast(`Failed to get withdrawable vaults: ${vaultsResult.error.readableMsg}`);
		return false;
	}
	const vaults = vaultsResult.value;
	// Early return if no vaults are selected
	if (vaults.length === 0) {
		errToast('No vaults selected for withdrawal.');
		return false;
	}

	return new Promise((resolve) => {
		handleWithdrawAllModal({
			open: true,
			vaults,
			onSubmit: async () => {
				try {
					// Validate that all vaults share the same raindex
					const raindex = vaults[0].raindex;
					const calldataResult = await vaultsList.getWithdrawCalldata();
					if (calldataResult.error) {
						throw new Error(
							`Failed to generate multicall calldata: ${calldataResult.error.readableMsg}`
						);
					}
					const calldata = calldataResult.value;

					const result = await handleTransactionConfirmationModal({
						open: true,
						modalTitle: `Withdrawing from ${vaults.length} vaults...`,
						args: {
							toAddress: raindex,
							chainId: vaults[0].chainId,
							calldata,
							onConfirm: async (txHash: Hex) => {
								manager.createVaultsWithdrawAllTransaction({
									raindexClient,
									vaults,
									txHash,
									queryKey: QKEY_VAULTS, // Invalidate all vaults
									chainId: vaults[0].chainId
								});
							}
						}
					});
					resolve(result.success);
				} catch (error) {
					const errorMsg = error instanceof Error ? error.message : 'Unknown error';
					errToast(`Failed to generate calldata for vault withdrawal: ${errorMsg}`);
					resolve(false);
				}
			}
		});
	});
}
