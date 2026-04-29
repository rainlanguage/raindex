import type { LayoutLoad } from './$types';
import { redirect } from '@sveltejs/kit';
import type { DotrainRegistry, NameAndDescriptionCfg } from '@rainlanguage/raindex';

type ParentData = {
	orderName: string;
	deployments: Map<string, NameAndDescriptionCfg>;
	registry: DotrainRegistry | null;
	orderDetail?: NameAndDescriptionCfg;
};

export const load: LayoutLoad = async ({ params, parent }) => {
	const { deploymentKey } = params;
	const { orderName, deployments, registry, orderDetail } = (await parent()) as ParentData;

	if (!registry || !deploymentKey) {
		throw redirect(307, '/deploy');
	}

	const deploymentDetails = deployments.get(deploymentKey);

	if (!deploymentDetails) {
		throw redirect(307, '/deploy');
	}

	return {
		deployment: {
			key: deploymentKey,
			name: deploymentDetails.name,
			description: deploymentDetails.description
		},
		orderName,
		orderDetail,
		registry,
		pageName: deploymentKey
	};
};
