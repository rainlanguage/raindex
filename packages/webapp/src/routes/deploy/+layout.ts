import type { LayoutLoad } from './$types';
import type { InvalidOrderDetail, ValidOrderDetail } from '@rainlanguage/ui-components';
import type { DotrainRegistry } from '@rainlanguage/raindex';

/**
 * Type defining the structure of the load function's return value,
 * including registry information and validation results.
 */
type LoadResult = {
	validOrders: ValidOrderDetail[];
	invalidOrders: InvalidOrderDetail[];
	registry: DotrainRegistry | null;
	error: string | null;
};

interface ParentData {
	registry?: DotrainRegistry | null;
}

export const load: LayoutLoad<LoadResult> = async ({ parent }) => {
	const parentData: ParentData = await parent();
	const registry = parentData.registry ?? null;

	if (!registry) {
		return {
			validOrders: [],
			invalidOrders: [],
			registry,
			error: 'Registry not loaded'
		};
	}

	try {
		const orderDetails = registry.getAllOrderDetails();
		if (orderDetails.error) {
			return {
				validOrders: [],
				invalidOrders: [],
				registry,
				error: orderDetails.error.readableMsg ?? orderDetails.error.msg
			};
		}

		const validOrders: ValidOrderDetail[] = Array.from(orderDetails.value.valid.entries()).map(
			([name, details]) => ({
				name,
				dotrain: registry.orders.get(name) ?? '',
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
			registry,
			error: null
		};
	} catch (error: unknown) {
		return {
			validOrders: [],
			invalidOrders: [],
			registry,
			error: error instanceof Error ? error.message : 'Unknown error occurred'
		};
	}
};
