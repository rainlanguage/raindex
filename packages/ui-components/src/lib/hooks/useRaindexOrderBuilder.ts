import { getContext } from 'svelte';
import { RaindexOrderBuilder } from '@rainlanguage/orderbook';
import { DeploymentStepsError, DeploymentStepsErrorCode } from '../errors/DeploymentStepsError';
export const RAINDEX_ORDER_BUILDER_CONTEXT_KEY = 'raindex-order-builder-context';

export function useRaindexOrderBuilder(): RaindexOrderBuilder {
	const builder = getContext<RaindexOrderBuilder>(RAINDEX_ORDER_BUILDER_CONTEXT_KEY);
	if (!builder) {
		DeploymentStepsError.catch(null, DeploymentStepsErrorCode.NO_BUILDER_PROVIDER);
	}
	return builder;
}
