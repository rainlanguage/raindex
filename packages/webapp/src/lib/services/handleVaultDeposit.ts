import type { Float, RaindexClient, RaindexVault } from "@rainlanguage/raindex";
import { type Hex } from "viem";
import type {
  TransactionManager,
  VaultActionModalProps,
  TransactionConfirmationProps,
} from "@rainlanguage/ui-components";

export interface VaultDepositHandlerDependencies {
  raindexClient: RaindexClient;
  vault: RaindexVault;
  handleDepositModal: (props: VaultActionModalProps) => void;
  handleTransactionConfirmationModal: (
    props: TransactionConfirmationProps,
  ) => void;
  errToast: (message: string) => void;
  manager: TransactionManager;
  account: Hex;
}

export type DepositArgs = VaultDepositHandlerDependencies & { amount: Float };

function executeDeposit(args: DepositArgs & { calldata: Hex }) {
  const {
    raindexClient,
    amount,
    vault,
    handleTransactionConfirmationModal,
    manager,
    calldata,
  } = args;
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
          amount,
        });
      },
      calldata,
    },
  });
}

export async function handleVaultDeposit(
  deps: VaultDepositHandlerDependencies,
): Promise<void> {
  const {
    vault,
    handleDepositModal,
    handleTransactionConfirmationModal,
    errToast,
    manager,
    account,
  } = deps;

  handleDepositModal({
    open: true,
    args: {
      vault,
      account,
    },
    onSubmit: async (amount: Float) => {
      const calldatasResult = await vault.getCalldatas(amount);
      if (calldatasResult.error) {
        return errToast(calldatasResult.error.msg);
      }
      const { approval, deposit } = calldatasResult.value;
      const depositArgs = { ...deps, amount, calldata: deposit };

      // When the raindex contract already has a sufficient allowance, `approval` is
      // undefined and no approval transaction is needed, so deposit directly.
      if (!approval) {
        return executeDeposit(depositArgs);
      }

      handleTransactionConfirmationModal({
        open: true,
        modalTitle: `Approving ${vault.token.symbol || "token"} spend`,
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
              entity: vault,
            });
            // Immediately invoke deposit after approval
            executeDeposit(depositArgs);
          },
          calldata: approval,
        },
      });
    },
  });
}
