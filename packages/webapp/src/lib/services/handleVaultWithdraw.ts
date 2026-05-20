import type { Float, RaindexClient, RaindexVault } from '@rainlanguage/raindex';
import { type Hex } from 'viem';
import type { TransactionManager } from '@rainlanguage/ui-components';
import type {
	VaultActionModalProps,
	TransactionConfirmationProps
} from '@rainlanguage/ui-components';

export interface VaultWithdrawHandlerDependencies {
	raindexClient: RaindexClient;
	vault: RaindexVault;
	handleWithdrawModal: (props: VaultActionModalProps) => void;
	handleTransactionConfirmationModal: (props: TransactionConfirmationProps) => void;
	errToast: (message: string) => void;
	manager: TransactionManager;
	account: Hex;
}

export async function handleVaultWithdraw(deps: VaultWithdrawHandlerDependencies): Promise<void> {
	const {
		raindexClient,
		vault,
		handleWithdrawModal,
		handleTransactionConfirmationModal,
		errToast,
		manager,
		account
	} = deps;
	handleWithdrawModal({
		open: true,
		args: {
			vault,
			account
		},
		onSubmit: async (amount: Float) => {
			let calldata: string;
			try {
				const calldataResult = await vault.getWithdrawCalldata(amount);
				if (calldataResult.error) {
					return errToast(calldataResult.error.msg);
				}
				calldata = calldataResult.value;
				handleTransactionConfirmationModal({
					open: true,
					modalTitle: `Withdrawing ${amount.format().value} ${vault.token.symbol}...`,
					args: {
						entity: vault,
						toAddress: vault.raindex,
						chainId: vault.chainId,
						onConfirm: (txHash: Hex) => {
							manager.createWithdrawTransaction({
								raindexClient,
								entity: vault,
								txHash,
								chainId: vault.chainId,
								queryKey: vault.id
							});
						},
						calldata
					}
				});
			} catch {
				return errToast('Failed to get calldata for vault withdrawal.');
			}
		}
	});
}
