import type { LayoutLoad } from './$types';
import type { InvalidOrderDetail, ValidOrderDetail } from '@rainlanguage/ui-components';
import type { DotrainRainlang } from '@rainlanguage/orderbook';

/**
 * Type defining the structure of the load function's return value,
 * including rainlang information and validation results.
 */
type LoadResult = {
	validOrders: ValidOrderDetail[];
	invalidOrders: InvalidOrderDetail[];
	rainlang: DotrainRainlang | null;
	error: string | null;
};

interface ParentData {
	rainlang?: DotrainRainlang | null;
}

export const load: LayoutLoad<LoadResult> = async ({ parent }) => {
	const parentData: ParentData = await parent();
	const rainlang = parentData.rainlang ?? null;

	if (!rainlang) {
		return {
			validOrders: [],
			invalidOrders: [],
			rainlang,
			error: 'Rainlang not loaded'
		};
	}

	try {
		const orderDetails = rainlang.getAllOrderDetails();
		if (orderDetails.error) {
			return {
				validOrders: [],
				invalidOrders: [],
				rainlang,
				error: orderDetails.error.readableMsg ?? orderDetails.error.msg
			};
		}

		const validOrders: ValidOrderDetail[] = Array.from(orderDetails.value.valid.entries()).map(
			([name, details]) => ({
				name,
				dotrain: rainlang.orders.get(name) ?? '',
				details
			})
		);
		const invalidOrders: InvalidOrderDetail[] = Array.from(
			orderDetails.value.invalid.entries()
		).map(([name, err]) => ({
			name,
			error: err.readableMsg ?? err.msg
		}));

		return {
			validOrders,
			invalidOrders,
			rainlang,
			error: null
		};
	} catch (error: unknown) {
		return {
			validOrders: [],
			invalidOrders: [],
			rainlang,
			error: error instanceof Error ? error.message : 'Unknown error occurred'
		};
	}
};
