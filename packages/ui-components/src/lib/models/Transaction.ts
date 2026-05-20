import type { Hex } from "viem";
import { waitForTransactionReceipt } from "@wagmi/core";
import {
  TransactionStatusMessage,
  TransactionStoreErrorMessage,
  type AwaitIndexingFn,
} from "$lib/types/transaction";
import type { TransactionArgs } from "$lib/types/transaction";
import type { Config } from "@wagmi/core";
import { writable, type Writable } from "svelte/store";
import type { ToastLink } from "$lib/types/toast";

/**
 * Represents the state of a transaction.
 * @typedef {Object} TransactionStoreState
 * @property {TransactionName} name - The user-friendly name of the transaction (e.g., "Order Removal").
 * @property {TransactionStatusMessage} status - The current status of the transaction (e.g., PENDING_RECEIPT, SUCCESS, ERROR).
 * @property {TransactionStoreErrorMessage} [errorDetails] - Optional error message if the transaction failed.
 * @property {Array<{link: string, label: string}>} links - An array of relevant links for the transaction (e.g., explorer link, link to the affected entity).
 */
export type TransactionStoreState = {
  name: string;
  status: TransactionStatusMessage;
  errorDetails?: TransactionStoreErrorMessage;
  links: {
    link: string;
    label: string;
  }[];
};

/**
 * Interface defining the structure of a transaction object.
 * @interface Transaction
 * @property {Writable<TransactionStoreState>} state - A writable store holding the current state of the transaction.
 */
export type Transaction = {
  readonly state: Writable<TransactionStoreState>;
};

/**
 * Manages the lifecycle of an individual transaction, including waiting for
 * blockchain confirmation (receipt) and subgraph indexing.
 * It exposes its state as a Svelte writable store.
 * @class TransactionStore
 * @implements {Transaction}
 */
export class TransactionStore implements Transaction {
  private name: string;
  private config: Config;
  private txHash: Hex;
  private links: ToastLink[];
  private onSuccess: () => void;
  private onError: () => void;
  // Optional indexing function called after receipt confirmation.
  // TransactionStore doesn't know what this function does - it just calls it.
  private awaitIndexingFn?: AwaitIndexingFn;
  public readonly state: Writable<TransactionStoreState>;

  /**
   * Creates a new TransactionStore instance.
   * @param {TransactionArgs & { config: Config }} args - Configuration arguments for the transaction, including the wagmi `Config`.
   * @param {() => void} onSuccess - Callback invoked when the transaction successfully completes (including indexing).
   * @param {() => void} onError - Callback invoked if the transaction fails at any stage.
   */
  constructor(
    args: TransactionArgs & { config: Config },
    onSuccess: () => void,
    onError: () => void,
  ) {
    this.config = args.config;
    this.txHash = args.txHash;
    this.name = args.name;
    this.links = args.toastLinks;
    this.state = writable<TransactionStoreState>({
      name: this.name,
      status: TransactionStatusMessage.IDLE,
      links: this.links,
    });
    this.awaitIndexingFn = args.awaitIndexingFn;
    this.onSuccess = onSuccess;
    this.onError = onError;
  }

  /**
   * Updates the internal Svelte store with new state values.
   * @param {Partial<TransactionStoreState>} partialState - An object containing the state properties to update.
   * @private
   */
  private updateState(partialState: Partial<TransactionStoreState>): void {
    this.state.update((currentState) => ({
      ...currentState,
      ...partialState,
    }));
  }

  /**
   * Executes the transaction and begins monitoring its status
   * @returns {Promise<void>}
   */
  public async execute(): Promise<void> {
    this.updateState({
      status: TransactionStatusMessage.PENDING_RECEIPT,
    });
    await this.waitForTxReceipt(this.txHash);
  }

  /**
   * Waits for the transaction receipt to be confirmed on the blockchain.
   * If an indexing function is provided, it will be called after receipt confirmation.
   * TransactionStore doesn't know what the indexing function does - it just calls it.
   * @param {Hex} hash - The transaction hash to monitor
   * @returns {Promise<void>}
   * @private
   */
  private async waitForTxReceipt(hash: Hex): Promise<void> {
    try {
      await waitForTransactionReceipt(this.config, { hash });

      if (this.awaitIndexingFn) {
        // Call the indexing function with the context it needs.
        // The function handles everything: updating state, calling callbacks, etc.
        await this.awaitIndexingFn({
          updateState: this.updateState.bind(this),
          onSuccess: this.onSuccess,
          onError: this.onError,
          links: this.links,
        });
      } else {
        // No indexing needed, mark as success immediately
        this.updateState({
          status: TransactionStatusMessage.SUCCESS,
        });
        return this.onSuccess();
      }
    } catch {
      this.updateState({
        status: TransactionStatusMessage.ERROR,
        errorDetails: TransactionStoreErrorMessage.RECEIPT_FAILED,
      });
      return this.onError();
    }
  }
}
