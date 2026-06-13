import { RegistryManager } from "../providers/registry/RegistryManager";

export async function loadRegistryUrl(
  url: string,
  registryManager: RegistryManager,
): Promise<void> {
  if (!url) {
    throw new Error("No URL provided");
  }

  if (!registryManager) {
    throw new Error("Registry manager is required");
  }

  try {
    // Persist the new registry URL and reload. The page-load (+layout.ts)
    // calls DotrainRegistry.new(url), which fetches and validates the
    // registry. Validating here as well would refetch the same registry and
    // settings files from GitHub, doubling the HTTP requests, so validation is
    // left to page-load.
    registryManager.setRegistry(url);
    window.location.reload();
  } catch (e) {
    const errorMessage =
      e instanceof Error ? e.message : "Failed to update registry URL";
    throw new Error(errorMessage);
  }
}
