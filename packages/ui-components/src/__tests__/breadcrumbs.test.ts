import { expect, test } from "vitest";
import { generateBreadcrumbs } from "../lib/utils/breadcrumbs";

test("generateBreadcrumbs splits a path into segments, excluding current page, with uppercase labels", () => {
  const path = "/my/cool/path/12345/abc/$$$az";
  const crumbs = generateBreadcrumbs(path);

  expect(crumbs.length).toEqual(5);
  expect(crumbs[0]).toEqual({
    label: "my",
    href: "/my",
  });
  expect(crumbs[1]).toEqual({
    label: "cool",
    href: "/my/cool",
  });
  expect(crumbs[2]).toEqual({
    label: "path",
    href: "/my/cool/path",
  });
  expect(crumbs[3]).toEqual({
    label: "12345",
    href: "/my/cool/path/12345",
  });
  expect(crumbs[4]).toEqual({
    label: "abc",
    href: "/my/cool/path/12345/abc",
  });
});
