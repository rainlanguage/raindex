export interface BreadCrumb {
  label: string;
  href: string;
}

export function generateBreadcrumbs(path: string): BreadCrumb[] {
  const crumbs = path.split("/");
  return crumbs
    .map((c, i) => ({
      label: c,
      href: crumbs.slice(0, i + 1).join("/"),
    }))
    .slice(1, -1);
}
