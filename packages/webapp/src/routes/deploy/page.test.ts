import { describe, beforeEach, it, expect, vi, type Mock } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import Page from './+page.svelte';
import { readable } from 'svelte/store';
import {
	useRegistry,
	type ValidOrderDetail,
	type InvalidOrderDetail
} from '@rainlanguage/ui-components';

const { mockPageStore } = await vi.hoisted(() => import('$lib/__mocks__/stores'));

vi.mock('$app/stores', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		page: mockPageStore
	};
});

const mockValidOrder1: ValidOrderDetail = {
	details: {
		name: 'Order One',
		description: 'This is the first valid order.',
		short_description: 'Valid 1'
	},
	name: 'order1.dotrain',
	dotrain: ';;'
};

const mockRegistry = vi.fn();
const mockIsCustomRegistry = vi.fn();

const mockValidOrder2: ValidOrderDetail = {
	details: {
		name: 'Order Two',
		description: 'Another valid order.',
		short_description: 'Valid 2'
	},
	name: 'order1.dotrain',
	dotrain: ';;'
};

const mockInvalidOrder1: InvalidOrderDetail = {
	name: 'invalidOrder.dotrain',
	error: 'Syntax error on line 1'
};

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		useRegistry: vi.fn()
	};
});

const mockGetCurrentRegistry = vi.fn().mockReturnValue(readable({}));

describe('Page Component', () => {
	const mockValidated = {
		validOrders: [mockValidOrder1, mockValidOrder2],
		invalidOrders: [mockInvalidOrder1]
	};

	beforeEach(() => {
		vi.resetAllMocks();
		(useRegistry as Mock).mockReturnValue(
			readable({
				getCurrentRegistry: mockGetCurrentRegistry,
				isCustomRegistry: mockIsCustomRegistry,
				subscribe: vi.fn()
			})
		);
		mockIsCustomRegistry.mockReturnValue(true);
		mockPageStore.reset();
	});

	it('should display error message when fetching orders fails', async () => {
		mockPageStore.mockSetSubscribeValue({
			data: {
				error: 'Failed to fetch registry dotrains'
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
			} as any
		});

		render(Page, {
			context: new Map([['$$_registry', mockRegistry]])
		});

		await waitFor(() => {
			const errorMessage = screen.getByTestId('error-message');
			expect(errorMessage).toBeInTheDocument();
			expect(errorMessage).toHaveTextContent('Failed to fetch registry dotrains');
		});
	});

	it('should display error message when validating orders fails', async () => {
		mockPageStore.mockSetSubscribeValue({
			data: {
				error: 'Failed to validate orders'
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
			} as unknown as any
		});

		render(Page, {
			context: new Map([['$$_registry', mockRegistry]])
		});

		await waitFor(() => {
			const errorMessage = screen.getByTestId('error-message');
			expect(errorMessage).toBeInTheDocument();
			expect(errorMessage).toHaveTextContent('Failed to validate orders');
		});
	});

	it('should display no orders found when no orders are available', async () => {
		mockPageStore.mockSetSubscribeValue({
			data: {
				error: null,
				validOrders: [],
				invalidOrders: []
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
			} as unknown as any
		});

		render(Page, {
			context: new Map([['$$_registry', mockRegistry]])
		});

		await waitFor(() => {
			expect(screen.getByText('No orders found')).toBeInTheDocument();
		});
	});

	it('should display valid orders when they are available', async () => {
		mockPageStore.mockSetSubscribeValue({
			data: {
				validOrders: mockValidated.validOrders,
				invalidOrders: []
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
			} as any
		});

		render(Page, {
			context: new Map([['$$_registry', mockRegistry]])
		});

		await waitFor(() => {
			expect(screen.getByTestId('valid-orders')).toBeInTheDocument();
		});
	});

	it('should display invalid orders when they are available', async () => {
		mockPageStore.mockSetSubscribeValue({
			data: {
				validOrders: [],
				invalidOrders: mockValidated.invalidOrders
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
			} as any
		});

		render(Page, {
			context: new Map([['$$_registry', mockRegistry]])
		});

		await waitFor(() => {
			expect(screen.getByTestId('invalid-orders')).toBeInTheDocument();
		});
	});

	it('should display valid and invalid orders when both are available', async () => {
		mockPageStore.mockSetSubscribeValue({
			data: {
				validOrders: mockValidated.validOrders,
				invalidOrders: mockValidated.invalidOrders
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
			} as any
		});

		render(Page, {
			context: new Map([['$$_registry', mockRegistry]])
		});

		await waitFor(() => {
			expect(screen.getByTestId('valid-orders')).toBeInTheDocument();
			expect(screen.getByTestId('invalid-orders')).toBeInTheDocument();
		});
	});
});
