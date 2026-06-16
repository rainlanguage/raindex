import type { LayoutLoad } from './$types';
import { redirect } from '@sveltejs/kit';
import type { DotrainRainlang, NameAndDescriptionCfg } from '@rainlanguage/orderbook';

type ParentData = {
	orderName: string;
	deployments: Map<string, NameAndDescriptionCfg>;
	rainlang: DotrainRainlang | null;
	orderDetail?: NameAndDescriptionCfg;
};

export const load: LayoutLoad = async ({ params, parent }) => {
	const { deploymentKey } = params;
	const { orderName, deployments, rainlang, orderDetail } = (await parent()) as ParentData;

	if (!rainlang || !deploymentKey) {
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
		rainlang,
		pageName: deploymentKey
	};
};
