import { getContext, setContext } from "svelte";
import { type Writable } from "svelte/store";
import type { ToastProps } from "$lib/types/toast";

export const TOASTS_KEY = "rain:ui-components:toasts";

/**
 * Retrieves the toasts store from Svelte's context
 */
export function getToastsContext(): Writable<ToastProps[]> {
  const toasts = getContext<Writable<ToastProps[]>>(TOASTS_KEY);
  if (!toasts) {
    throw new Error(
      "No toasts context found. Did you forget to wrap your component with ToastProvider?",
    );
  }
  return toasts;
}

/**
 * Sets the toasts store in Svelte's context
 *
 * @param {Writable<ToastProps[]>} toasts - The writable store containing all active toast notifications
 * @returns {void}
 */
export function setToastsContext(toasts: Writable<ToastProps[]>) {
  setContext(TOASTS_KEY, toasts);
}
