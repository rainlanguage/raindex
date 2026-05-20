import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { getContext } from "svelte";
import {
  useRaindexClient,
  RAINDEX_CLIENT_CONTEXT_KEY,
} from "./useRaindexClient";
import {
  DeploymentStepsError,
  DeploymentStepsErrorCode,
} from "../errors/DeploymentStepsError";

vi.mock("svelte", () => ({
  getContext: vi.fn(),
}));

vi.spyOn(DeploymentStepsError, "catch");

describe("useRaindexClient hook", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it("should return Raindex client when available", () => {
    const mockRaindexClient = {
      someMethod: vi.fn(),
      someProperty: "value",
    };

    vi.mocked(getContext).mockReturnValue(mockRaindexClient);

    const result = useRaindexClient();

    expect(getContext).toHaveBeenCalledWith(RAINDEX_CLIENT_CONTEXT_KEY);
    expect(result).toBe(mockRaindexClient);
  });

  it("should call DeploymentStepsError.catch when Raindex client context is not available", () => {
    vi.mocked(getContext).mockReturnValue(null);

    useRaindexClient();

    expect(DeploymentStepsError.catch).toHaveBeenCalledWith(
      null,
      DeploymentStepsErrorCode.NO_RAINDEX_CLIENT_PROVIDER,
    );
  });
});
