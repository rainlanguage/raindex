import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  handleVaultDeposit,
  type VaultDepositHandlerDependencies,
} from "../lib/services/handleVaultDeposit";
import {
  Float,
  type RaindexClient,
  type RaindexVault,
} from "@rainlanguage/raindex";
import type { Hex } from "viem";
import { waitFor } from "@testing-library/svelte";
import type { TransactionManager } from "@rainlanguage/ui-components";

// Mocks
const mockHandleDepositModal = vi.fn();
const mockHandleTransactionConfirmationModal = vi.fn();
const mockErrToast = vi.fn();
const mockCreateDepositTransaction = vi.fn();
const mockCreateApprovalTransaction = vi.fn();

const mockManager = {
  createDepositTransaction: mockCreateDepositTransaction,
  createApprovalTransaction: mockCreateApprovalTransaction,
};

const mockRaindexClient = {} as unknown as RaindexClient;

const mockVault = {
  id: "0xvaultid",
  token: {
    address: "0xtokenaddress",
    symbol: "TEST",
  },
  getCalldatas: vi.fn(),
} as unknown as RaindexVault;

const mockDeps: VaultDepositHandlerDependencies = {
  raindexClient: mockRaindexClient,
  vault: mockVault,
  account: "0xaccount" as Hex,
  handleDepositModal: mockHandleDepositModal,
  handleTransactionConfirmationModal: mockHandleTransactionConfirmationModal,
  errToast: mockErrToast,
  manager: mockManager as unknown as TransactionManager,
};

describe("handleVaultDeposit", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should call handleDepositModal with correct arguments", async () => {
    await handleVaultDeposit(mockDeps);
    expect(mockHandleDepositModal).toHaveBeenCalledWith({
      open: true,
      args: {
        vault: mockVault,
        account: mockDeps.account,
      },
      onSubmit: expect.any(Function),
    });
  });

  describe("onSubmit callback from handleDepositModal", () => {
    const mockAmount = Float.parse("100").value as Float;
    const mockApprovalCalldata = "0xapprovalcalldata" as Hex;
    const mockDepositCalldata = "0xdepositcalldata" as Hex;
    const mockWithdrawCalldata = "0xwithdrawcalldata" as Hex;
    const mockTxHashApproval = "0xtxhashapproval" as Hex;
    const mockTxHashDeposit = "0xtxhashdeposit" as Hex;

    beforeEach(async () => {
      await handleVaultDeposit(mockDeps);
    });

    it("should show error toast if getCalldatas returns an error", async () => {
      vi.mocked(mockVault.getCalldatas).mockResolvedValue({
        error: {
          msg: "Calldata error",
          readableMsg: "Calldata error readable",
        },
        value: undefined,
      });

      const onSubmitCall = mockHandleDepositModal.mock.calls[0][0].onSubmit;
      await onSubmitCall(mockAmount);

      expect(mockVault.getCalldatas).toHaveBeenCalledWith(mockAmount);
      expect(mockErrToast).toHaveBeenCalledWith("Calldata error");
      expect(mockHandleTransactionConfirmationModal).not.toHaveBeenCalled();
    });

    it("should deposit directly without an approval tx when no approval is needed", async () => {
      // `approval` is undefined when the raindex contract already has a sufficient
      // allowance, so no approval transaction should be sent.
      vi.mocked(mockVault.getCalldatas).mockResolvedValue({
        value: {
          approval: undefined,
          deposit: mockDepositCalldata,
          withdraw: mockWithdrawCalldata,
        },
        error: undefined,
      });

      const onSubmitCall = mockHandleDepositModal.mock.calls[0][0].onSubmit;
      await onSubmitCall(mockAmount);

      expect(mockVault.getCalldatas).toHaveBeenCalledTimes(1);
      expect(mockVault.getCalldatas).toHaveBeenCalledWith(mockAmount);
      // Exactly one confirmation modal (the deposit), no approval modal.
      expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(1);
      expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledWith({
        open: true,
        modalTitle: "Depositing 100 TEST",
        closeOnConfirm: false,
        args: expect.objectContaining({
          entity: mockVault,
          toAddress: mockVault.raindex,
          chainId: mockVault.chainId,
          onConfirm: expect.any(Function),
          calldata: mockDepositCalldata,
        }),
      });
      expect(mockCreateApprovalTransaction).not.toHaveBeenCalled();

      // Confirming the deposit creates the deposit transaction.
      const onDepositConfirmCall =
        mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
      onDepositConfirmCall(mockTxHashDeposit);
      expect(mockCreateDepositTransaction).toHaveBeenCalledWith({
        raindexClient: mockRaindexClient,
        txHash: mockTxHashDeposit,
        chainId: mockVault.chainId,
        queryKey: mockVault.id,
        entity: mockVault,
        amount: mockAmount,
      });
    });

    it("should handle approval and then deposit when approval is needed", async () => {
      vi.mocked(mockVault.getCalldatas).mockResolvedValue({
        value: {
          approval: mockApprovalCalldata,
          deposit: mockDepositCalldata,
          withdraw: mockWithdrawCalldata,
        },
        error: undefined,
      });

      const onSubmitCall = mockHandleDepositModal.mock.calls[0][0].onSubmit;
      await onSubmitCall(mockAmount);

      // Only the on-chain allowance is read once; no separate deposit calldata call.
      expect(mockVault.getCalldatas).toHaveBeenCalledTimes(1);
      expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(1);
      expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
        1,
        {
          open: true,
          modalTitle: "Approving TEST spend",
          closeOnConfirm: true,
          args: {
            entity: mockVault,
            toAddress: mockVault.token.address as Hex,
            chainId: mockVault.chainId,
            onConfirm: expect.any(Function),
            calldata: mockApprovalCalldata,
          },
        },
      );

      // Simulate approval confirmation
      const onApprovalConfirmCall =
        mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
      onApprovalConfirmCall(mockTxHashApproval);

      expect(mockCreateApprovalTransaction).toHaveBeenCalledWith({
        txHash: mockTxHashApproval,
        chainId: mockVault.chainId,
        queryKey: mockVault.id,
        entity: mockVault,
      });

      await waitFor(() => {
        expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(2);
        expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
          2,
          {
            open: true,
            modalTitle: "Depositing 100 TEST",
            closeOnConfirm: false,
            args: {
              entity: mockVault,
              toAddress: mockVault.raindex,
              chainId: mockVault.chainId,
              onConfirm: expect.any(Function),
              calldata: mockDepositCalldata,
            },
          },
        );
      });

      const onDepositConfirmCall =
        mockHandleTransactionConfirmationModal.mock.calls[1][0].args.onConfirm;
      onDepositConfirmCall(mockTxHashDeposit);

      expect(mockCreateDepositTransaction).toHaveBeenCalledWith({
        raindexClient: mockRaindexClient,
        txHash: mockTxHashDeposit,
        chainId: mockVault.chainId,
        queryKey: mockVault.id,
        entity: mockVault,
        amount: mockAmount,
      });
    });
  });
});
