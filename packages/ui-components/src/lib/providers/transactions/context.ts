import { getContext, setContext } from "svelte";
import { TransactionManager } from "./TransactionManager";

/**
 * The context key used to store and retrieve the TransactionManager instance
 */
export const TRANSACTION_MANAGER_CONTEXT_KEY =
  "rain:ui-components:transactionManager";

/**
 * Sets the TransactionManager instance in Svelte's context
 *
 * @param {TransactionManager} manager - The TransactionManager instance to store in context
 * @returns {void}
 */
export function setTransactionManagerContext(manager: TransactionManager) {
  setContext(TRANSACTION_MANAGER_CONTEXT_KEY, manager);
}

/**
 * Retrieves the TransactionManager instance from Svelte's context
 *
 * @returns {TransactionManager} The TransactionManager instance
 * @throws {Error} If no TransactionManager is found in context
 */
export function getTransactionManagerContext(): TransactionManager {
  const manager = getContext<TransactionManager | undefined>(
    TRANSACTION_MANAGER_CONTEXT_KEY,
  );
  if (!manager) {
    throw new Error(
      "TransactionManager not found. Did you forget to setTransactionManagerContext?",
    );
  }
  return manager;
}
