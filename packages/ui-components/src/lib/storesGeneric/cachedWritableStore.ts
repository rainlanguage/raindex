import { writable } from "svelte/store";

/**
 * Creates a writable Svelte store that persists its value to localStorage.
 *
 * @template T - The type of the value stored in the store
 * @param {string} key - The localStorage key used to store the value
 * @param {T} defaultValue - The default value to use when no value is found in localStorage
 * @param {function(T): string} serialize - Function to convert the store value to a string for storage
 * @param {function(string): T} deserialize - Function to convert the stored string back to the original type
 * @returns {import('svelte/store').Writable<T>} A writable store that automatically syncs with localStorage
 *
 * @example
 * // Create a store for a boolean value
 * const darkMode = cachedWritableStore(
 *   'darkMode',
 *   false,
 *   value => JSON.stringify(value),
 *   str => JSON.parse(str)
 * );
 *
 * // Create a store for a complex object
 * const userPreferences = cachedWritableStore(
 *   'userPrefs',
 *   { theme: 'light', fontSize: 14 },
 *   value => JSON.stringify(value),
 *   str => JSON.parse(str)
 * );
 */
export function cachedWritableStore<T>(
  key: string,
  defaultValue: T,
  serialize: (value: T) => string,
  deserialize: (serialized: string) => T,
) {
  const getCache = () => {
    try {
      const cached = localStorage.getItem(key);
      return cached !== null ? deserialize(cached) : defaultValue;
    } catch {
      return defaultValue;
    }
  };
  const setCache = (value?: T) => {
    try {
      if (value !== undefined) {
        localStorage.setItem(key, serialize(value));
      } else {
        localStorage.removeItem(key);
      }
    } catch {
      // Silently ignore localStorage errors to allow the application to function
      // without persistence in environments where localStorage is unavailable
    }
  };

  const data = writable<T>(getCache());

  data.subscribe((value) => {
    setCache(value);
  });

  return data;
}

export const cachedWritableString = (key: string, defaultValue = "") =>
  cachedWritableStore<string>(
    key,
    defaultValue,
    (v) => v,
    (v) => v,
  );
export const cachedWritableInt = (key: string, defaultValue = 0) =>
  cachedWritableStore<number>(
    key,
    defaultValue,
    (v) => v.toString(),
    (v) => {
      const parsed = Number.parseInt(v);
      return isNaN(parsed) ? defaultValue : parsed;
    },
  );
/**
 * Creates a writable store that can hold an optional value of type T and persists to localStorage.
 *
 * @template T - The type of the value stored
 * @param {string} key - The localStorage key to use for persistence
 * @param {T | undefined} defaultValue - The default value if nothing is found in localStorage
 * @param {function} serialize - Function to convert the value to a string for storage
 * @param {function} deserialize - Function to convert the stored string back to a value
 * @returns A writable store that persists to localStorage and can hold undefined values
 */
export const cachedWritableOptionalStore = <T>(
  key: string,
  defaultValue: T | undefined = undefined,
  serialize: (value: T) => string,
  deserialize: (serialized: string) => T,
) =>
  cachedWritableStore<T | undefined>(
    key,
    defaultValue,
    (v) => (v !== undefined ? serialize(v) : ""),
    (v) => (v !== "" ? deserialize(v) : undefined),
  );

/**
 * Creates a writable store that can hold an optional number value and persists to localStorage.
 *
 * @param {string} key - The localStorage key to use for persistence
 * @param {number | undefined} defaultValue - The default value if nothing is found in localStorage
 * @returns A writable store that persists to localStorage and can hold an optional number
 */
export const cachedWritableIntOptional = (
  key: string,
  defaultValue = undefined,
) =>
  cachedWritableOptionalStore<number>(
    key,
    defaultValue,
    (v) => v.toString(),
    (v) => {
      const parsed = Number.parseInt(v);
      return isNaN(parsed) ? (defaultValue ?? 0) : parsed;
    },
  );

/**
 * Creates a writable store that can hold an optional string value and persists to localStorage.
 *
 * @param {string} key - The localStorage key to use for persistence
 * @param {string | undefined} defaultValue - The default value if nothing is found in localStorage
 * @returns A writable store that persists to localStorage and can hold an optional string
 */
export const cachedWritableStringOptional = (
  key: string,
  defaultValue = undefined,
) =>
  cachedWritableOptionalStore<string>(
    key,
    defaultValue,
    (v) => v,
    (v) => v,
  );
