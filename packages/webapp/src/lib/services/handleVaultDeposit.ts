import type { Float, RaindexClient, RaindexVault } from '@rainlanguage/raindex';
import { type Hex } from 'viem';
import type {
	TransactionManager,
	VaultActionModalProps,
	TransactionConfirmationProps
} from '@rainlanguage/ui-components';

export interface VaultDepositHandlerDependencies {
	raindexClient: RaindexClient;
	vault: RaindexVault;
	handleDepositModal: (props: VaultActionModalProps) => void;
	handleTransactionConfirmationModal: (props: TransactionConfirmationProps) => void;
	errToast: (message: string) => void;
	manager: TransactionManager;
	account: Hex;
}

export type DepositArgs = VaultDepositHandlerDependencies & { amount: Float };

async function executeDeposit(args: DepositArgs) {
	const { raindexClient, amount, vault, handleTransactionConfirmationModal, errToast, manager } =
		args;
	const calldataResult = await vault.getDepositCalldata(amount);
	if (calldataResult.error) {
		return errToast(calldataResult.error.msg);
	} else if (calldataResult.value) {
		handleTransactionConfirmationModal({
			open: true,
			modalTitle: `Depositing ${amount.format().value} ${vault.token.symbol}`,
			closeOnConfirm: false,
			args: {
				entity: vault,
				toAddress: vault.raindex,
				chainId: vault.chainId,
				onConfirm: (txHash: Hex) => {
					manager.createDepositTransaction({
						raindexClient,
						txHash,
						chainId: vault.chainId,
						queryKey: vault.id,
						entity: vault,
						amount
					});
				},
				calldata: calldataResult.value
			}
		});
	}
}

export async function handleVaultDeposit(deps: VaultDepositHandlerDependencies): Promise<void> {
	const { vault, handleDepositModal, handleTransactionConfirmationModal, manager, account } = deps;

	handleDepositModal({
		open: true,
		args: {
			vault,
			account
		},
		onSubmit: async (amount: Float) => {
			const depositArgs = { ...deps, amount };
			const approvalResult = await vault.getApprovalCalldata(amount);
			if (approvalResult.error) {
				// If getting approval calldata fails, immediately invoke deposit
				await executeDeposit(depositArgs);
			} else if (approvalResult.value) {
				handleTransactionConfirmationModal({
					open: true,
					modalTitle: `Approving ${vault.token.symbol || 'token'} spend`,
					closeOnConfirm: true,
					args: {
						entity: vault,
						toAddress: vault.token.address as Hex,
						chainId: vault.chainId,
						onConfirm: (txHash: Hex) => {
							manager.createApprovalTransaction({
								txHash,
								chainId: vault.chainId,
								queryKey: vault.id,
								entity: vault
							});
							// Immediately invoke deposit after approval
							executeDeposit(depositArgs);
						},
						calldata: approvalResult.value
					}
				});
			}
		}
	});
}
