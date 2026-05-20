import type { LayoutLoad } from './$types';
import { redirect } from '@sveltejs/kit';
import type { NameAndDescriptionCfg, DotrainRegistry } from '@rainlanguage/raindex';
import type { InvalidOrderDetail, ValidOrderDetail } from '@rainlanguage/ui-components';

type ParentData = {
	validOrders: ValidOrderDetail[];
	invalidOrders: InvalidOrderDetail[];
	registry: DotrainRegistry | null;
};

export const load: LayoutLoad = async ({ params, parent }) => {
	const { orderName } = params;
	const { validOrders, registry } = (await parent()) as ParentData;

	if (!registry) {
		throw redirect(307, '/deploy');
	}

	const orderDetail = validOrders.find((detail) => detail.name === orderName)?.details;
	if (!orderDetail) {
		throw redirect(307, '/deploy');
	}

	const deploymentsResult = registry.getDeploymentDetails(orderName);
	if (deploymentsResult.error) {
		throw redirect(307, '/deploy');
	}

	return {
		orderName,
		orderDetail: orderDetail as NameAndDescriptionCfg,
		deployments: deploymentsResult.value,
		registry,
		pageName: orderName
	};
};
