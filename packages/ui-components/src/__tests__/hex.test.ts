import { expect, test } from "vitest";
import { HEX_INPUT_REGEX } from "../lib/utils/hex";

test("HEX_INPUT_REGEX matches user typing hex input", () => {
  expect(HEX_INPUT_REGEX.test("a")).toBeTruthy();
  expect(HEX_INPUT_REGEX.test("ab")).toBeTruthy();
  expect(HEX_INPUT_REGEX.test("abc")).toBeTruthy();
  expect(HEX_INPUT_REGEX.test("abcdef1234567890")).toBeTruthy();
  expect(HEX_INPUT_REGEX.test("1")).toBeTruthy();
  expect(HEX_INPUT_REGEX.test("12")).toBeTruthy();
  expect(HEX_INPUT_REGEX.test("123")).toBeTruthy();
  expect(HEX_INPUT_REGEX.test("1234567890abcdef")).toBeTruthy();
});

test('HEX_INPUT_REGEX matches user typing hex input prefixed by "0x"', () => {
  expect(HEX_INPUT_REGEX.test("0")).toBeTruthy();
  expect(HEX_INPUT_REGEX.test("0x")).toBeTruthy();
  expect(HEX_INPUT_REGEX.test("0xa")).toBeTruthy();
  expect(HEX_INPUT_REGEX.test("0xab")).toBeTruthy();
  expect(HEX_INPUT_REGEX.test("0xabc")).toBeTruthy();
  expect(HEX_INPUT_REGEX.test("0xabcdef1234567890")).toBeTruthy();
  expect(HEX_INPUT_REGEX.test("0x1")).toBeTruthy();
  expect(HEX_INPUT_REGEX.test("0x12")).toBeTruthy();
  expect(HEX_INPUT_REGEX.test("0x123")).toBeTruthy();
  expect(HEX_INPUT_REGEX.test("0x1234567890abcdef")).toBeTruthy();
});

test("HEX_INPUT_REGEX does not match user typing invalid hex input", () => {
  expect(HEX_INPUT_REGEX.test("0xx")).toBeFalsy();
  expect(HEX_INPUT_REGEX.test("0xg")).toBeFalsy();
  expect(HEX_INPUT_REGEX.test("0xag")).toBeFalsy();
  expect(HEX_INPUT_REGEX.test("0xabg")).toBeFalsy();
  expect(HEX_INPUT_REGEX.test("0xabcdef1234567890g")).toBeFalsy();
  expect(HEX_INPUT_REGEX.test("0x1g")).toBeFalsy();
  expect(HEX_INPUT_REGEX.test("0x12g")).toBeFalsy();
  expect(HEX_INPUT_REGEX.test("0x123g")).toBeFalsy();
  expect(HEX_INPUT_REGEX.test("0x1234567890abcdefg")).toBeFalsy();

  expect(HEX_INPUT_REGEX.test("g")).toBeFalsy();
  expect(HEX_INPUT_REGEX.test("ag")).toBeFalsy();
  expect(HEX_INPUT_REGEX.test("abg")).toBeFalsy();
  expect(HEX_INPUT_REGEX.test("abcdef1234567890g")).toBeFalsy();
  expect(HEX_INPUT_REGEX.test("1g")).toBeFalsy();
  expect(HEX_INPUT_REGEX.test("12g")).toBeFalsy();
  expect(HEX_INPUT_REGEX.test("123g")).toBeFalsy();
  expect(HEX_INPUT_REGEX.test("1234567890abcdefg")).toBeFalsy();
});
