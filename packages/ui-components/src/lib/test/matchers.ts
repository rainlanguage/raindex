import { expect as vitestExpect } from "vitest";
import * as matchers from "@testing-library/jest-dom/matchers";
vitestExpect.extend(matchers);

export const expect = vitestExpect;

declare module "vitest" {
  // vitest instead `@vitest/expect`
  // eslint-disable-next-line @typescript-eslint/no-empty-object-type, @typescript-eslint/no-explicit-any
  interface JestAssertion<T = any>
    extends matchers.TestingLibraryMatchers<
      ReturnType<typeof expect.stringContaining>,
      T
    > {}
}
