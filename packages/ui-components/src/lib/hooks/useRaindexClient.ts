import { getContext } from 'svelte';
import { RaindexClient } from '@rainlanguage/raindex';
import { DeploymentStepsError, DeploymentStepsErrorCode } from '../errors/DeploymentStepsError';
export const RAINDEX_CLIENT_CONTEXT_KEY = 'raindex-client-context';

export function useRaindexClient(): RaindexClient {
	const raindexClient = getContext<RaindexClient>(RAINDEX_CLIENT_CONTEXT_KEY);
	if (!raindexClient) {
		DeploymentStepsError.catch(null, DeploymentStepsErrorCode.NO_RAINDEX_CLIENT_PROVIDER);
	}
	return raindexClient;
}
