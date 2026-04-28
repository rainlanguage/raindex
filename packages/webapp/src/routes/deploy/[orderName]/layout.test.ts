import { describe, it, expect, vi, beforeEach } from 'vitest';
import { load } from './+layout';
import { redirect } from '@sveltejs/kit';

vi.mock('@sveltejs/kit', () => ({
	redirect: vi.fn()
}));

describe('Layout load function', () => {
	const mockorderName = 'test-order';
	const mockorderDetail = {
		name: 'Test Order',
		description: 'This is a test order',
		config: {}
	};
	const mockDeployments = new Map([
		['test-deployment', { name: 'Test Deployment', description: 'desc' }]
	]);

	const mockParent = vi.fn();

	beforeEach(() => {
		vi.resetAllMocks();

		mockParent.mockResolvedValue({
			validOrders: [
				{
					name: mockorderName,
					details: mockorderDetail
				},
				{
					name: 'other-order',
					details: { name: 'Other', description: 'Other order', config: {} }
				}
			],
			invalidOrders: [],
			registry: {
				getDeploymentDetails: vi.fn().mockReturnValue({ value: mockDeployments })
			}
		});
	});

	it('should load order details successfully', async () => {
		const result = await load({
			params: { orderName: mockorderName },
			parent: mockParent
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		} as any);

		expect(mockParent).toHaveBeenCalled();
		expect(result).toEqual({
			orderName: mockorderName,
			orderDetail: mockorderDetail,
			deployments: mockDeployments,
			registry: {
				getDeploymentDetails: expect.any(Function)
			},
			pageName: mockorderName
		});
	});

	it('should redirect if order name is not found in validOrders', async () => {
		try {
			await load({
				params: { orderName: 'non-existent-order' },
				parent: mockParent
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
			} as any);
		} catch {
			expect(redirect).toHaveBeenCalledWith(307, '/deploy');
		}
	});

	it('should redirect if order details are not found in validOrders', async () => {
		mockParent.mockResolvedValue({
			validOrders: [
				{ name: 'incomplete-order', details: undefined },
				{ name: 'other-order', details: { name: 'Other', description: '', config: {} } }
			],
			invalidOrders: [],
			registry: {
				getDeploymentDetails: vi.fn().mockReturnValue({ value: mockDeployments })
			}
		});

		try {
			await load({
				params: { orderName: 'incomplete-order' },
				parent: mockParent
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
			} as any);
			expect(redirect).toHaveBeenCalled();
		} catch {
			expect(redirect).toHaveBeenCalledWith(307, '/deploy');
		}
	});
});
