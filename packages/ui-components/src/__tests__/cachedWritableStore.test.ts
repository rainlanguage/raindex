import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { get } from "svelte/store";
import {
  cachedWritableStore,
  cachedWritableString,
  cachedWritableInt,
  cachedWritableOptionalStore,
  cachedWritableIntOptional,
  cachedWritableStringOptional,
} from "../lib/storesGeneric/cachedWritableStore";

describe("cachedWritableStore", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.restoreAllMocks();
  });

  afterEach(() => {
    localStorage.clear();
    vi.restoreAllMocks();
  });

  describe("cachedWritableStore (core)", () => {
    it("initializes from localStorage when a value is present", () => {
      localStorage.setItem("key", "42");
      const store = cachedWritableStore<number>(
        "key",
        0,
        (v) => v.toString(),
        (v) => Number(v),
      );
      // Must deserialize the cached value, not fall back to the default.
      expect(get(store)).toBe(42);
    });

    it("initializes from the default when localStorage has no value", () => {
      const store = cachedWritableStore<number>(
        "key",
        7,
        (v) => v.toString(),
        (v) => Number(v),
      );
      expect(get(store)).toBe(7);
    });

    it("distinguishes a stored empty string from a missing key", () => {
      // getItem returns "" (not null) -> must deserialize "" rather than default.
      localStorage.setItem("key", "");
      const store = cachedWritableStore<string>(
        "key",
        "DEFAULT",
        (v) => v,
        (v) => `deserialized:${v}`,
      );
      expect(get(store)).toBe("deserialized:");
    });

    it("falls back to the default when reading localStorage throws", () => {
      vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
        throw new Error("boom");
      });
      const store = cachedWritableStore<number>(
        "key",
        99,
        (v) => v.toString(),
        (v) => Number(v),
      );
      expect(get(store)).toBe(99);
    });

    it("persists the serialized value to localStorage on set", () => {
      const store = cachedWritableStore<number>(
        "key",
        0,
        (v) => `n=${v}`,
        (v) => Number(v.replace("n=", "")),
      );
      store.set(5);
      expect(localStorage.getItem("key")).toBe("n=5");
    });

    it("persists the initial value to localStorage immediately on creation", () => {
      cachedWritableStore<number>(
        "key",
        3,
        (v) => `init-${v}`,
        (v) => Number(v),
      );
      // The subscribe callback runs synchronously on creation.
      expect(localStorage.getItem("key")).toBe("init-3");
    });

    it("removes the localStorage key when the value is set to undefined", () => {
      const store = cachedWritableStore<number | undefined>(
        "key",
        1,
        (v) => String(v),
        (v) => Number(v),
      );
      expect(localStorage.getItem("key")).toBe("1");
      store.set(undefined);
      // undefined branch must call removeItem, not setItem("undefined").
      expect(localStorage.getItem("key")).toBe(null);
    });

    it("uses the serialize function (not toString) when persisting", () => {
      const serialize = vi.fn((v: number) => `S${v}`);
      const store = cachedWritableStore<number>("key", 0, serialize, (v) =>
        Number(v.slice(1)),
      );
      serialize.mockClear();
      store.set(8);
      expect(serialize).toHaveBeenCalledWith(8);
      expect(localStorage.getItem("key")).toBe("S8");
    });

    it("swallows errors thrown while writing to localStorage", () => {
      const store = cachedWritableStore<number>(
        "key",
        0,
        (v) => v.toString(),
        (v) => Number(v),
      );
      vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
        throw new Error("quota");
      });
      // Must not throw despite setItem failing.
      expect(() => store.set(123)).not.toThrow();
      // Store value still updates in memory.
      expect(get(store)).toBe(123);
    });
  });

  describe("cachedWritableString", () => {
    it("defaults to empty string and round-trips identity", () => {
      const store = cachedWritableString("strkey");
      expect(get(store)).toBe("");
      store.set("hello");
      expect(localStorage.getItem("strkey")).toBe("hello");
    });

    it("honors an explicit default value", () => {
      const store = cachedWritableString("strkey", "fallback");
      expect(get(store)).toBe("fallback");
    });

    it("reads back a previously stored string verbatim", () => {
      localStorage.setItem("strkey", "stored-value");
      const store = cachedWritableString("strkey", "fallback");
      expect(get(store)).toBe("stored-value");
    });
  });

  describe("cachedWritableInt", () => {
    it("defaults to 0 and serializes via toString", () => {
      const store = cachedWritableInt("intkey");
      expect(get(store)).toBe(0);
      store.set(15);
      expect(localStorage.getItem("intkey")).toBe("15");
    });

    it("honors an explicit default value", () => {
      const store = cachedWritableInt("intkey", 100);
      expect(get(store)).toBe(100);
    });

    it("parses a stored integer", () => {
      localStorage.setItem("intkey", "55");
      const store = cachedWritableInt("intkey", 9);
      expect(get(store)).toBe(55);
    });

    it("falls back to the default when the stored value is not a number", () => {
      localStorage.setItem("intkey", "not-a-number");
      const store = cachedWritableInt("intkey", 9);
      // parseInt -> NaN -> must return the default, not NaN.
      expect(get(store)).toBe(9);
    });
  });

  describe("cachedWritableOptionalStore", () => {
    it("removes the key on undefined without calling the inner serialize", () => {
      const serialize = vi.fn((v: string) => `S:${v}`);
      const store = cachedWritableOptionalStore<string>(
        "optkey",
        "init",
        serialize,
        (v) => `D:${v}`,
      );
      store.set(undefined);
      // undefined store value -> setCache removes the key (getItem -> null).
      expect(localStorage.getItem("optkey")).toBe(null);
      // The optional wrapper short-circuits undefined; inner serialize never sees it.
      expect(serialize).not.toHaveBeenCalledWith(
        undefined as unknown as string,
      );
    });

    it("serializes a defined value through the inner serialize", () => {
      const store = cachedWritableOptionalStore<string>(
        "optkey",
        undefined,
        (v) => `S:${v}`,
        (v) => v,
      );
      store.set("x");
      expect(localStorage.getItem("optkey")).toBe("S:x");
    });

    it("deserializes empty string to undefined", () => {
      localStorage.setItem("optkey", "");
      const store = cachedWritableOptionalStore<string>(
        "optkey",
        "fallback",
        (v) => v,
        (v) => `D:${v}`,
      );
      // "" -> undefined (skip inner deserialize), not "D:".
      expect(get(store)).toBe(undefined);
    });

    it("deserializes a non-empty string through the inner deserialize", () => {
      localStorage.setItem("optkey", "raw");
      const store = cachedWritableOptionalStore<string>(
        "optkey",
        undefined,
        (v) => v,
        (v) => `D:${v}`,
      );
      expect(get(store)).toBe("D:raw");
    });
  });

  describe("cachedWritableIntOptional", () => {
    it("parses a stored integer", () => {
      localStorage.setItem("ioptkey", "77");
      const store = cachedWritableIntOptional("ioptkey", 5);
      expect(get(store)).toBe(77);
    });

    it("falls back to the provided default when not a number", () => {
      localStorage.setItem("ioptkey", "xyz");
      const store = cachedWritableIntOptional("ioptkey", 5);
      // NaN -> defaultValue ?? 0 -> 5.
      expect(get(store)).toBe(5);
    });

    it("falls back to 0 when not a number and no default is given", () => {
      localStorage.setItem("ioptkey", "xyz");
      const store = cachedWritableIntOptional("ioptkey");
      // NaN -> (undefined ?? 0) -> 0.
      expect(get(store)).toBe(0);
    });

    it("treats a missing key as undefined and persists nothing", () => {
      const store = cachedWritableIntOptional("ioptkey");
      // defaultValue is undefined; missing key deserializes to undefined.
      expect(get(store)).toBe(undefined);
      // An undefined initial value is not written: setCache removes the key.
      expect(localStorage.getItem("ioptkey")).toBe(null);
    });
  });

  describe("cachedWritableStringOptional", () => {
    it("defaults to undefined when no value is stored", () => {
      const store = cachedWritableStringOptional("soptkey");
      expect(get(store)).toBe(undefined);
    });

    it("round-trips a stored non-empty string", () => {
      localStorage.setItem("soptkey", "abc");
      const store = cachedWritableStringOptional("soptkey");
      expect(get(store)).toBe("abc");
    });

    it("persists a set value and clears it on undefined", () => {
      const store = cachedWritableStringOptional("soptkey");
      store.set("def");
      expect(localStorage.getItem("soptkey")).toBe("def");
      store.set(undefined);
      // undefined store value -> setCache removes the key.
      expect(localStorage.getItem("soptkey")).toBe(null);
    });
  });
});
