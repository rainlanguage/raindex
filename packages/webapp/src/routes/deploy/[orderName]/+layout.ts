import type { LayoutLoad } from './$types';
import { redirect } from '@sveltejs/kit';
import type { NameAndDescriptionCfg, DotrainRainlang } from '@rainlanguage/orderbook';
import type { InvalidOrderDetail, ValidOrderDetail } from '@rainlanguage/ui-components';

type ParentData = {
	validOrders: ValidOrderDetail[];
	invalidOrders: InvalidOrderDetail[];
	rainlang: DotrainRainlang | null;
};

export const load: LayoutLoad = async ({ params, parent }) => {
	const { orderName } = params;
	const { validOrders, rainlang } = (await parent()) as ParentData;

	if (!rainlang) {
		throw redirect(307, '/deploy');
	}

	const orderDetail = validOrders.find((detail) => detail.name === orderName)?.details;
	if (!orderDetail) {
		throw redirect(307, '/deploy');
	}

	const deploymentsResult = rainlang.getDeploymentDetails(orderName);
	if (deploymentsResult.error) {
		throw redirect(307, '/deploy');
	}

	return {
		orderName,
		orderDetail: orderDetail as NameAndDescriptionCfg,
		deployments: deploymentsResult.value,
		rainlang,
		pageName: orderName
	};
};
