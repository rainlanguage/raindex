import { RegistryManager } from '../providers/registry/RegistryManager';
import { DotrainRegistry } from '@rainlanguage/raindex';

export async function loadRegistryUrl(
	url: string,
	registryManager: RegistryManager
): Promise<void> {
	if (!url) {
		throw new Error('No URL provided');
	}

	if (!registryManager) {
		throw new Error('Registry manager is required');
	}

	try {
		const validationResult = await DotrainRegistry.validate(url);
		if (validationResult.error) {
			throw new Error(validationResult.error.readableMsg);
		}
		registryManager.setRegistry(url);
		window.location.reload();
	} catch (e) {
		const errorMessage = e instanceof Error ? e.message : 'Failed to update registry URL';
		throw new Error(errorMessage);
	}
}
