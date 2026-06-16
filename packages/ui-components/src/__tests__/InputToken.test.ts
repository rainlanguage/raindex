import { render, fireEvent } from "@testing-library/svelte";
import InputToken from "../lib/components/input/InputToken.svelte";

describe("InputToken", () => {
  it("renders with initial values", () => {
    const address = "0xc0D477556c25C9d67E1f57245C7453DA776B51cf";
    const decimals = 10;
    const { getByTestId } = render(InputToken, {
      props: { address, decimals },
    });

    const input = getByTestId("token-address").querySelector("input");
    expect(input?.value).toBe("0xc0D477556c25C9d67E1f57245C7453DA776B51cf");
    const decimalsInput = getByTestId("token-decimals-input").querySelector(
      "input",
    );
    expect(decimalsInput?.value).toBe("10");
  });

  it("shows error for invalid address", async () => {
    const address = "abc";
    const decimals = 0;
    const { getByTestId, getByText } = render(InputToken, {
      props: { address, decimals },
    });

    const addressInput = getByTestId("token-address").querySelector(
      "input",
    ) as HTMLInputElement;
    await fireEvent.input(addressInput, {
      target: { value: "invalidAddress" },
    });

    expect(getByText("Invalid address")).toBeInTheDocument();
  });

  it("does not show error for valid address", async () => {
    const address = "";
    const decimals = 0;
    const { getByTestId, queryByText } = render(InputToken, {
      props: { address, decimals },
    });

    const addressInput = getByTestId("token-address").querySelector(
      "input",
    ) as HTMLInputElement;
    await fireEvent.input(addressInput, {
      target: { value: "0xc0D477556c25C9d67E1f57245C7453DA776B51cf" },
    });

    expect(queryByText("Invalid address")).toBeNull();
  });
});
