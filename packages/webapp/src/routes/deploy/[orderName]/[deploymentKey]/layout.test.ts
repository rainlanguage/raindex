import { describe, it, expect, vi, beforeEach } from 'vitest';
import { load } from './+layout';

const deploymentMap = new Map([
	['test-deployment', { name: 'Test Deployment', description: 'This is a test deployment' }]
]);

describe('Layout load function', () => {
	const mockParent = vi.fn();

	beforeEach(() => {
		vi.resetAllMocks();
		mockParent.mockResolvedValue({
			orderName: 'test-order',
			deployments: deploymentMap,
			registry: {} // non-null placeholder
		});
	});

	it('should load deployment details successfully', async () => {
		const result = await load({
			params: { deploymentKey: 'test-deployment', orderName: 'test-order' },
			parent: mockParent
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		} as any);

		expect(result).toEqual({
			deployment: {
				key: 'test-deployment',
				name: 'Test Deployment',
				description: 'This is a test deployment'
			},
			orderName: 'test-order',
			orderDetail: undefined,
			registry: {},
			pageName: 'test-deployment'
		});
	});

	it('should redirect when deployment is missing', async () => {
		await expect(
			load({
				params: { deploymentKey: 'missing', orderName: 'test-order' },
				parent: mockParent
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
			} as any)
		).rejects.toMatchObject({ status: 307, location: '/deploy' });
	});
});
