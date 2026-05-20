import type { Hex } from "viem";
import { get } from "svelte/store";
import { isAddress, isAddressEqual } from "viem";
import { getAccountContext } from "./context";
import { readable } from "svelte/store";

/**
 * Hook to access wallet account information from context
 * Must be used within a component that is a child of WalletProvider
 */
export function useAccount() {
  /**
   * The account store containing the current wallet address (as a Hex string) or null if not connected.
   * This is a readable Svelte store that can be subscribed to for reactive updates.
   * @type {import('svelte/store').Readable<Hex | null>}
   */
  const account = getAccountContext();

  /**
   * Checks if the provided address matches the currently connected account.
   * Returns false if no account is connected or if the provided address is invalid.
   */
  const matchesAccount = (otherAddress: string): boolean => {
    if (!otherAddress) return false;
    const currentAccount = get(account);
    if (!currentAccount) {
      return false;
    }

    if (
      isAddress(currentAccount) &&
      isAddress(otherAddress) &&
      isAddressEqual(currentAccount, otherAddress)
    ) {
      return true;
    }

    return false;
  };

  return {
    account,
    matchesAccount,
  };
}

if (import.meta.vitest) {
  const { describe, it, expect, vi, beforeEach } = import.meta.vitest;

  vi.mock("viem", async () => {
    const actual = await vi.importActual("viem");
    return {
      ...actual,
      isAddress: vi.fn(),
      isAddressEqual: vi.fn(),
    };
  });

  vi.mock("./context", () => ({
    getAccountContext: vi.fn(),
  }));

  vi.mock("svelte/store", async () => {
    const actual = await vi.importActual("svelte/store");
    return {
      ...actual,
      get: vi.fn(),
    };
  });

  describe("useAccount", () => {
    const mockGetAccountContext = vi.mocked(getAccountContext);
    const mockGet = vi.mocked(get);
    const mockIsAddress = vi.mocked(isAddress);
    const mockIsAddressEqual = vi.mocked(isAddressEqual);

    beforeEach(() => {
      vi.clearAllMocks();
    });

    it("should return account wrapped in an object", () => {
      const mockAccountStore = readable("0x123" as Hex);
      mockGetAccountContext.mockReturnValue(mockAccountStore);

      const result = useAccount();

      expect(mockGetAccountContext).toHaveBeenCalled();
      expect(result.account).toBe(mockAccountStore);
      expect(result.matchesAccount).toBeInstanceOf(Function);
    });

    describe("matchesAccount", () => {
      const mockAccountStore = readable("0x123" as Hex);
      const currentAccount = "0x123" as Hex;
      const testAddress1 = "0x123" as Hex;
      const testAddress2 = "0xdef" as Hex;
      const invalidAddress = "invalid";

      beforeEach(() => {
        mockGetAccountContext.mockReturnValue(mockAccountStore);
      });

      it("should return true if addresses are valid and equal", () => {
        // Setup mocks
        mockGet.mockReturnValue(currentAccount);
        mockIsAddress.mockReturnValue(true);
        mockIsAddressEqual.mockReturnValue(true);

        const { matchesAccount } = useAccount();
        const result = matchesAccount(testAddress1);

        expect(mockGet).toHaveBeenCalledWith(mockAccountStore);
        expect(mockIsAddress).toHaveBeenCalledWith(currentAccount);
        expect(mockIsAddress).toHaveBeenCalledWith(testAddress1);
        expect(mockIsAddressEqual).toHaveBeenCalledWith(
          currentAccount,
          testAddress1,
        );
        expect(result).toBe(true);
      });

      it("should return false if addresses are valid but not equal", () => {
        // Setup mocks
        mockGet.mockReturnValue(currentAccount);
        mockIsAddress.mockReturnValue(true);
        mockIsAddressEqual.mockReturnValue(false);

        const { matchesAccount } = useAccount();
        const result = matchesAccount(testAddress2);

        expect(mockGet).toHaveBeenCalledWith(mockAccountStore);
        expect(mockIsAddress).toHaveBeenCalledWith(currentAccount);
        expect(mockIsAddress).toHaveBeenCalledWith(testAddress2);
        expect(mockIsAddressEqual).toHaveBeenCalledWith(
          currentAccount,
          testAddress2,
        );
        expect(result).toBe(false);
      });

      it("should return false if current account is not set", () => {
        // Setup mocks
        mockGet.mockReturnValue(null);

        const { matchesAccount } = useAccount();
        const result = matchesAccount(testAddress1);

        expect(mockGet).toHaveBeenCalledWith(mockAccountStore);
        expect(mockIsAddress).not.toHaveBeenCalled();
        expect(mockIsAddressEqual).not.toHaveBeenCalled();
        expect(result).toBe(false);
      });
      it("should return false if provided address is invalid", () => {
        // Setup mocks
        mockGet.mockReturnValue(currentAccount);

        // This is crucial: we need to ensure short-circuit evaluation works correctly
        mockIsAddress.mockImplementation((address) => {
          return address !== invalidAddress; // Only the invalid address returns false
        });

        // This should never be called due to short-circuit evaluation
        mockIsAddressEqual.mockReturnValue(false);

        const { matchesAccount } = useAccount();
        const result = matchesAccount(invalidAddress);

        expect(mockGet).toHaveBeenCalledWith(mockAccountStore);
        expect(mockIsAddress).toHaveBeenCalledWith(currentAccount);
        expect(mockIsAddress).toHaveBeenCalledWith(invalidAddress);
        expect(mockIsAddressEqual).not.toHaveBeenCalled(); // This should now pass
        expect(result).toBe(false);
      });
      it("should return false if provided address is null", () => {
        // Setup mocks
        mockGet.mockReturnValue(currentAccount);

        const { matchesAccount } = useAccount();
        const result = matchesAccount(null as unknown as string);

        expect(mockIsAddress).not.toHaveBeenCalled();
        expect(mockIsAddressEqual).not.toHaveBeenCalled();
        expect(result).toBe(false);
      });
    });
  });
}
