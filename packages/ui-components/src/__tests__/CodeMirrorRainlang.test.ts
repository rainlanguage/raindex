import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, waitFor, fireEvent } from "@testing-library/svelte";
import CodeMirrorRainlang from "../lib/components/CodeMirrorRainlang.svelte";
import type { RaindexOrder } from "@rainlanguage/raindex";
import { writable } from "svelte/store";

vi.mock("codemirror-rainlang", () => ({
  RainlangLR: vi.fn(),
}));

vi.mock("svelte-codemirror-editor", async () => {
  const mockCodeMirror = (
    await import("../lib/__mocks__/MockCodeMirrorEditor.svelte")
  ).default;
  return { default: mockCodeMirror };
});

describe("CodeMirrorRainlang", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should use order.rainlang when order prop is provided", async () => {
    const mockOrder: RaindexOrder = {
      rainlang: "mocked rainlang text",
    } as RaindexOrder;

    const screen = render(CodeMirrorRainlang, {
      props: {
        order: mockOrder,
        codeMirrorTheme: writable({}),
        codeMirrorDisabled: false,
        codeMirrorStyles: {},
      },
    });

    await waitFor(() => {
      const mockComponent = screen.getByTestId("mock-component");
      expect(mockComponent).toHaveAttribute("value", "mocked rainlang text");
    });
  });

  it("should use rainlangText when no order is provided", async () => {
    const testText = "test rainlang text";

    const screen = render(CodeMirrorRainlang, {
      props: {
        order: undefined,
        rainlangText: testText,
        codeMirrorTheme: writable({}),
        codeMirrorDisabled: false,
        codeMirrorStyles: {},
      },
    });

    await waitFor(() => {
      const mockComponent = screen.getByTestId("mock-component");
      expect(mockComponent).toHaveAttribute("value", "test rainlang text");
    });
  });

  it("should pass through disabled prop", async () => {
    const mockOrder: RaindexOrder = {
      rainlang: "mocked rainlang text",
    } as RaindexOrder;

    const screen = render(CodeMirrorRainlang, {
      props: {
        order: mockOrder,
        codeMirrorTheme: writable({}),
        codeMirrorDisabled: false,
        codeMirrorStyles: {},
      },
    });

    await waitFor(() => {
      const mockComponent = screen.getByTestId("mock-component");
      expect(mockComponent).toHaveAttribute("readonly", "false");
    });
  });

  it("strips trailing whitespace from the editor value on blur", async () => {
    const screen = render(CodeMirrorRainlang, {
      props: {
        rainlangText: "_a: 1;   \n_b: 2;\t\n\n   \n",
        codeMirrorTheme: writable({}),
        codeMirrorDisabled: false,
        codeMirrorStyles: {},
      },
    });

    const editor = await screen.findByTestId("mock-component");
    // Initial value retains the trailing whitespace until blur.
    expect(editor).toHaveAttribute("value", "_a: 1;   \n_b: 2;\t\n\n   \n");

    await fireEvent.blur(editor);

    await waitFor(() => {
      expect(screen.getByTestId("mock-component")).toHaveAttribute(
        "value",
        "_a: 1;\n_b: 2;",
      );
    });
  });

  it("calls onSave with the stripped text on blur", async () => {
    const onSave = vi.fn();
    const screen = render(CodeMirrorRainlang, {
      props: {
        rainlangText: "_a: 1;  \n_b: 2; \n",
        codeMirrorTheme: writable({}),
        codeMirrorDisabled: false,
        codeMirrorStyles: {},
        onSave,
      },
    });

    const editor = await screen.findByTestId("mock-component");
    await fireEvent.blur(editor);

    expect(onSave).toHaveBeenCalledTimes(1);
    expect(onSave).toHaveBeenCalledWith("_a: 1;\n_b: 2;");
  });

  it("does not strip whitespace on render before blur", async () => {
    const onSave = vi.fn();
    const screen = render(CodeMirrorRainlang, {
      props: {
        rainlangText: "_a: 1;   ",
        codeMirrorTheme: writable({}),
        codeMirrorDisabled: false,
        codeMirrorStyles: {},
        onSave,
      },
    });

    const editor = await screen.findByTestId("mock-component");
    expect(editor).toHaveAttribute("value", "_a: 1;   ");
    expect(onSave).not.toHaveBeenCalled();
  });
});
