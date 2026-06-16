import { getToastsContext } from "./context";
import type { ToastProps } from "$lib/types/toast";

/**
 * Hook for managing toast notifications in the application.
 * Provides functionality to add, remove, and access toast notifications.
 *
 * @returns {Object} An object containing:
 *   - toasts: Writable store containing all active toast notifications
 *   - addToast: Function to add a new toast notification
 *   - removeToast: Function to remove a toast notification by index
 */
export function useToasts() {
  const toasts = getToastsContext();

  /**
   * Removes a toast notification by its index from the toasts store
   *
   * @param {number} index - The index of the toast to remove
   * @returns {void}
   */
  const removeToast = (index: number) => {
    toasts.update((toasts) => {
      if (index < 0 || index >= toasts.length) {
        return toasts;
      }
      return toasts.filter((_, i) => i !== index);
    });
  };

  /**
   * Adds a new toast notification to the toasts store
   *
   * @param {ToastProps} toast - The toast configuration object containing:
   *   - message: The text to display in the toast
   *   - type: The type of toast (success, error, warning, info)
   *   - color: The color theme of the toast (green, red, yellow, blue)
   *   - links: Optional array of links to display in the toast
   * @returns {void}
   */
  const addToast = (toast: ToastProps) => {
    toasts.update((toasts) => {
      const updatedToasts = [...toasts, toast];
      return updatedToasts;
    });
  };

  /**
   * Adds a standardized error toast notification
   *
   * @param message - The error message to display
   * @param detail - The detail of the error
   */
  const errToast = (message: string, detail?: string) => {
    addToast({
      message,
      detail,
      type: "error",
      color: "red",
    });
  };

  return {
    toasts,
    addToast,
    removeToast,
    errToast,
  };
}
