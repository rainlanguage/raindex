import { describe, beforeEach, it, expect, vi, type Mock } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import Page from './+page.svelte';
import { readable } from 'svelte/store';
import {
	useRainlang,
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

const mockRainlang = vi.fn();
const mockIsCustomRainlang = vi.fn();

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
		useRainlang: vi.fn()
	};
});

const mockGetCurrentRainlang = vi.fn().mockReturnValue(readable({}));

describe('Page Component', () => {
	const mockValidated = {
		validOrders: [mockValidOrder1, mockValidOrder2],
		invalidOrders: [mockInvalidOrder1]
	};

	beforeEach(() => {
		vi.resetAllMocks();
		(useRainlang as Mock).mockReturnValue(
			readable({
				getCurrentRainlang: mockGetCurrentRainlang,
				isCustomRainlang: mockIsCustomRainlang,
				subscribe: vi.fn()
			})
		);
		mockIsCustomRainlang.mockReturnValue(true);
		mockPageStore.reset();
	});

	it('should display error message when fetching orders fails', async () => {
		mockPageStore.mockSetSubscribeValue({
			data: {
				error: 'Failed to fetch rainlang dotrains'
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
			} as any
		});

		render(Page, {
			context: new Map([['$$_rainlang', mockRainlang]])
		});

		await waitFor(() => {
			const errorMessage = screen.getByTestId('error-message');
			expect(errorMessage).toBeInTheDocument();
			expect(errorMessage).toHaveTextContent('Failed to fetch rainlang dotrains');
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
			context: new Map([['$$_rainlang', mockRainlang]])
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
			context: new Map([['$$_rainlang', mockRainlang]])
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
			context: new Map([['$$_rainlang', mockRainlang]])
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
			context: new Map([['$$_rainlang', mockRainlang]])
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
			context: new Map([['$$_rainlang', mockRainlang]])
		});

		await waitFor(() => {
			expect(screen.getByTestId('valid-orders')).toBeInTheDocument();
			expect(screen.getByTestId('invalid-orders')).toBeInTheDocument();
		});
	});
});
