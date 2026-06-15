import { render, screen } from "@testing-library/svelte";
import { test, expect } from "vitest";
import { get } from "svelte/store";
import MockComponent from "../lib/__mocks__/MockComponent.svelte";
import { props } from "../lib/__mocks__/MockComponent";
import MockComponentTest from "./MockComponent.test.svelte";

test("defaults data-testid to mock-component when no testid prop is given", () => {
  render(MockComponent);
  expect(screen.getByTestId("mock-component")).toBeInTheDocument();
});

test("uses the testid prop as the data-testid", () => {
  render(MockComponent, { props: { testid: "custom-testid" } });
  expect(screen.getByTestId("custom-testid")).toBeInTheDocument();
  expect(screen.queryByTestId("mock-component")).not.toBeInTheDocument();
});

test("spreads remaining props onto the element as attributes", () => {
  render(MockComponent, { props: { testid: "with-attrs", source: "hello" } });
  const el = screen.getByTestId("with-attrs");
  expect(el.getAttribute("source")).toBe("hello");
  // testid is consumed for data-testid and not duplicated as a `testid` attr.
  expect(el.getAttribute("testid")).toBeNull();
});

test("mirrors the received props (excluding testid) into the shared store", () => {
  const createSeries = () => "series";
  render(MockComponent, {
    props: { testid: "store-instance", loading: true, createSeries },
  });
  const stored = get(props) as Record<string, unknown>;
  expect(stored.loading).toBe(true);
  expect(stored.createSeries).toBe(createSeries);
  expect(stored.testid).toBeUndefined();
});

test("mounts more than one instance with distinct testids without crashing", () => {
  // Rendering more than one MockComponent in a single tree is supported.
  // Distinct testids let each instance be queried individually.
  render(MockComponentTest);

  const first = screen.getByTestId("first");
  const second = screen.getByTestId("second");

  expect(first).toBeInTheDocument();
  expect(second).toBeInTheDocument();
  expect(first).not.toBe(second);

  // The spread props are applied per instance, distinct between the two.
  expect(first.getAttribute("source")).toBe("a");
  expect(second.getAttribute("source")).toBe("b");

  // Slot content is rendered inside the matching instance.
  expect(first).toHaveTextContent("first slot");
  expect(second).toHaveTextContent("second slot");
});
