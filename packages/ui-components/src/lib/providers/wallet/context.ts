import { getContext, setContext } from "svelte";
import { readable } from "svelte/store";
import type { Account } from "../../types/account";

export const ACCOUNT_KEY = "account_key";

/**
 * Retrieves the account store directly from Svelte's context
 */
export const getAccountContext = (): Account => {
  const account = getContext<Account>(ACCOUNT_KEY);
  if (!account) {
    throw new Error(
      "No account was found in Svelte context. Did you forget to wrap your component with WalletProvider?",
    );
  }
  return account;
};

/**
 * Sets the account store in Svelte's context
 */
export const setAccountContext = (account: Account) => {
  setContext(ACCOUNT_KEY, account);
};

if (import.meta.vitest) {
  const { describe, it, expect, vi, beforeEach } = import.meta.vitest;

  vi.mock("svelte", async (importOriginal) => ({
    ...((await importOriginal()) as object),
    getContext: vi.fn(),
  }));

  describe("getAccountContext", () => {
    const mockGetContext = vi.mocked(getContext);

    beforeEach(() => {
      mockGetContext.mockReset();
    });

    it("should return the account from context when it exists", () => {
      const mockAccount = readable("0x456");

      mockGetContext.mockImplementation((key) => {
        if (key === ACCOUNT_KEY) return mockAccount;
        return undefined;
      });

      const result = getAccountContext();
      expect(mockGetContext).toHaveBeenCalledWith(ACCOUNT_KEY);
      expect(result).toEqual(mockAccount);
    });

    it("should throw an error when account is not in context", () => {
      mockGetContext.mockReturnValue(undefined);

      expect(() => getAccountContext()).toThrow(
        "No account was found in Svelte context. Did you forget to wrap your component with WalletProvider?",
      );
    });
  });
}
