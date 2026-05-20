import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { writable } from "svelte/store";
import { useToasts } from "$lib/providers/toasts/useToasts";
import type { ToastProps } from "$lib/types/toast";
import { getToastsContext } from "$lib/providers/toasts/context";

vi.mock("$lib/providers/toasts/context", () => ({
  getToastsContext: vi.fn(),
}));

describe("useToasts", () => {
  let toastsStore: ReturnType<typeof writable<ToastProps[]>>;

  const getStoreValue = () => {
    let value: ToastProps[] = [];
    toastsStore.subscribe((val) => {
      value = val;
    });
    return value;
  };

  beforeEach(() => {
    toastsStore = writable<ToastProps[]>([]);
    vi.mocked(getToastsContext).mockReturnValue(toastsStore);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it("should return the toasts store and functions", () => {
    const result = useToasts();

    expect(result.toasts).toBe(toastsStore);
    expect(typeof result.addToast).toBe("function");
    expect(typeof result.removeToast).toBe("function");
    expect(typeof result.errToast).toBe("function");
  });

  describe("errToast", () => {
    it("should add an error toast with the given message", () => {
      const { errToast } = useToasts();
      const message = "Test error message";
      errToast(message);

      expect(getStoreValue()).toEqual([
        {
          message,
          type: "error",
          color: "red",
        },
      ]);
    });

    it("should add an error toast with the given message and detail", () => {
      const { errToast } = useToasts();
      const message = "Test error message with detail";
      const detail = "This is some extra detail.";
      errToast(message, detail);

      expect(getStoreValue()).toEqual([
        {
          message,
          detail,
          type: "error",
          color: "red",
        },
      ]);
    });

    it("should add an error toast with the given message and undefined detail", () => {
      const { errToast } = useToasts();
      const message = "Test error message with undefined detail";
      errToast(message, undefined);

      expect(getStoreValue()).toEqual([
        {
          message,
          detail: undefined,
          type: "error",
          color: "red",
        },
      ]);
    });
  });

  describe("addToast", () => {
    it("should add a toast to the store", () => {
      const { addToast } = useToasts();
      const testToast: ToastProps = {
        message: "Test Toast",
        type: "info",
        color: "green",
        links: [],
      };

      addToast(testToast);
      expect(getStoreValue()).toEqual([testToast]);
    });

    it("should add multiple toasts in sequence", () => {
      const { addToast } = useToasts();
      const toast1: ToastProps = {
        message: "Toast 1",
        type: "info",
        color: "green",
        links: [],
      };
      const toast2: ToastProps = {
        message: "Toast 2",
        type: "success",
        color: "blue",
        links: [],
      };

      addToast(toast1);
      addToast(toast2);

      expect(getStoreValue()).toEqual([toast1, toast2]);
    });

    it("should add a toast with minimal properties", () => {
      const { addToast } = useToasts();
      const minimalToast: ToastProps = {
        message: "Minimal Toast",
        type: "warning",
        color: "yellow",
      };

      addToast(minimalToast);
      expect(getStoreValue()).toEqual([minimalToast]);
    });
  });

  describe("removeToast", () => {
    it("should remove a toast at the specified index", () => {
      const { addToast, removeToast } = useToasts();
      const toast1: ToastProps = {
        message: "Toast 1",
        type: "info",
        color: "green",
      };
      const toast2: ToastProps = {
        message: "Toast 2",
        type: "success",
        color: "blue",
      };

      addToast(toast1);
      addToast(toast2);
      removeToast(0);

      expect(getStoreValue()).toEqual([toast2]);
    });

    it("should not modify the store when removing with an invalid index", () => {
      const { addToast, removeToast } = useToasts();
      const toast: ToastProps = {
        message: "Test Toast",
        type: "info",
        color: "green",
      };

      addToast(toast);
      removeToast(-1);
      removeToast(1);

      expect(getStoreValue()).toEqual([toast]);
    });
  });
});
