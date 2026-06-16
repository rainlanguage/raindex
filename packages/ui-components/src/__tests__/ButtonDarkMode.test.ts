import { render, screen, fireEvent } from "@testing-library/svelte";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { writable, get, type Writable } from "svelte/store";
import ButtonDarkMode from "../lib/components/ButtonDarkMode.svelte";

vi.mock("flowbite-svelte", async (importOriginal) => {
  const original = await importOriginal<typeof import("flowbite-svelte")>();
  return {
    ...original,
    DarkMode: (await import("../lib/__mocks__/MockComponent.svelte")).default,
  };
});

describe("ButtonDarkMode.svelte", () => {
  let mockColorThemeStore: Writable<string | undefined>;

  beforeEach(() => {
    mockColorThemeStore = writable<string | undefined>(undefined);
    document.documentElement.classList.remove("dark");
  });

  it("renders the button and the DarkMode component", () => {
    render(ButtonDarkMode, { props: { colorTheme: mockColorThemeStore } });

    expect(screen.getByRole("button")).toBeInTheDocument();
    expect(screen.getByTestId("mock-component")).toBeInTheDocument();
  });

  it("shows hover title and aria-label on the dark mode toggle button", () => {
    render(ButtonDarkMode, { props: { colorTheme: mockColorThemeStore } });

    const button = screen.getByTestId("dark-mode-toggle");
    expect(button).toHaveAttribute("title", "Toggle dark mode");
    expect(button).toHaveAttribute("aria-label", "Toggle dark mode");
  });

  it('sets colorTheme to "light" when toggled in light mode', async () => {
    render(ButtonDarkMode, { props: { colorTheme: mockColorThemeStore } });
    const button = screen.getByRole("button");

    expect(document.documentElement.classList.contains("dark")).toBe(false);
    expect(get(mockColorThemeStore)).toBeUndefined();

    await fireEvent.click(button);

    expect(get(mockColorThemeStore)).toBe("light");
    expect(document.documentElement.classList.contains("dark")).toBe(false);
  });

  it('sets colorTheme to "dark" when clicked in dark mode', async () => {
    document.documentElement.classList.add("dark");

    render(ButtonDarkMode, { props: { colorTheme: mockColorThemeStore } });
    const button = screen.getByRole("button");

    expect(document.documentElement.classList.contains("dark")).toBe(true);
    expect(get(mockColorThemeStore)).toBeUndefined();

    await fireEvent.click(button);

    expect(get(mockColorThemeStore)).toBe("dark");
    expect(document.documentElement.classList.contains("dark")).toBe(true);
  });
});
