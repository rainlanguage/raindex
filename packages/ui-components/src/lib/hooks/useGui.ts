import { getContext } from 'svelte';
import { DotrainOrderGui } from '@rainlanguage/raindex';
import { DeploymentStepsError, DeploymentStepsErrorCode } from '../errors/DeploymentStepsError';
export const GUI_CONTEXT_KEY = 'gui-context';

export function useGui(): DotrainOrderGui {
	const gui = getContext<DotrainOrderGui>(GUI_CONTEXT_KEY);
	if (!gui) {
		DeploymentStepsError.catch(null, DeploymentStepsErrorCode.NO_GUI_PROVIDER);
	}
	return gui;
}
