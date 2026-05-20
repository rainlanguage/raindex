import { getTransactionManagerContext } from "./context";

/**
 * Hook for accessing the transaction manager in the application.
 * Provides access to the transactions store for monitoring transaction states.
 *
 * This hook connects to the TransactionManager context which handles order removal
 * transactions and related operations.
 *
 * @returns An object containing the transactions store that can be subscribed to
 */
export function useTransactions() {
  const manager = getTransactionManagerContext();

  const transactions = manager.getTransactions();

  return {
    manager,
    transactions,
  };
}
