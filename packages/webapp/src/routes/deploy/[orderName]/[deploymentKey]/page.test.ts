import { render, screen, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';
import DeployPage from './+page.svelte';
import { useAccount, useToasts, useTransactions } from '@rainlanguage/ui-components';
import { readable, writable } from 'svelte/store';

const { mockPageStore } = await vi.hoisted(() => import('@rainlanguage/ui-components'));
const { mockedGoto } = await vi.hoisted(() => ({ mockedGoto: vi.fn() }));

const { mockConnectedStore, mockAppKitModalStore } = await vi.hoisted(
	() => import('$lib/__mocks__/stores')
);

vi.mock('$app/stores', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		page: mockPageStore
	};
});

vi.mock('$app/navigation', () => ({
	goto: mockedGoto
}));

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	const mockDeploymentSteps = (await import('$lib/__mocks__/MockComponent.svelte')).default;
	return {
		...((await importOriginal()) as object),
		DeploymentSteps: mockDeploymentSteps,
		useTransactions: vi.fn(),
		useAccount: vi.fn(),
		useToasts: vi.fn()
	};
});

vi.mock('$lib/stores/wagmi', () => ({
	connected: mockConnectedStore,
	appKitModal: mockAppKitModalStore
}));

vi.mock('$lib/services/modal', () => ({
	handleDisclaimerModal: vi.fn(),
	handleTransactionConfirmationModal: vi.fn()
}));

vi.mock('$lib/services/handleAddOrder', () => ({
	handleAddOrder: vi.fn()
}));

describe('DeployPage', () => {
	const mockRainlang = {
		getGui: vi.fn()
	};

	beforeEach(() => {
		vi.clearAllMocks();
		mockPageStore.reset();
		mockRainlang.getGui.mockReset();

		vi.mocked(useAccount).mockReturnValue({
			account: writable('0x123'),
			matchesAccount: vi.fn()
		});
		vi.mocked(useToasts).mockReturnValue({
			removeToast: vi.fn(),
			toasts: writable([]),
			addToast: vi.fn(),
			errToast: vi.fn()
		});
		vi.mocked(useTransactions).mockReturnValue({
			// @ts-expect-error simple object
			manager: writable({}),
			transactions: readable()
		});
		mockRainlang.getGui.mockResolvedValue({ value: {} });
	});

	afterEach(() => {
		vi.resetAllMocks();
	});

	it('should call rainlang.getGui with correct parameters when data exists', async () => {
		const mockDeploymentKey = 'test-key';
		const mockStateFromUrl = 'some-state';

		mockPageStore.mockSetSubscribeValue({
			data: {
				orderName: 'order-one',
				deployment: { key: mockDeploymentKey, name: 'Deployment', description: 'desc' },
				orderDetail: { name: 'Order', description: 'desc' },
				rainlang: mockRainlang
			},
			url: new URL(`http://localhost:3000/deploy?state=${mockStateFromUrl}`)
		});

		render(DeployPage);

		await vi.waitFor(() => {
			expect(mockRainlang.getGui).toHaveBeenCalledWith(
				'order-one',
				mockDeploymentKey,
				mockStateFromUrl,
				expect.any(Function)
			);
		});
	});

	it('should redirect to /deploy if rainlang or deployment is missing', async () => {
		vi.useFakeTimers();

		mockPageStore.mockSetSubscribeValue({
			data: {
				orderName: 'order-one',
				deployment: null,
				orderDetail: { name: 'Order', description: 'desc' },
				rainlang: null
			},
			url: new URL('http://localhost:3000/deploy/order/key')
		});

		render(DeployPage);

		expect(screen.getByText(/Deployment not found/i)).toBeInTheDocument();

		vi.advanceTimersByTime(5000);

		await vi.runAllTimersAsync();

		expect(mockedGoto).toHaveBeenCalledWith('/deploy');

		vi.useRealTimers();
	});

	it('should show error message when GUI initialization fails', async () => {
		const errorMessage = 'Failed to initialize GUI';

		mockRainlang.getGui.mockResolvedValue({
			error: { readableMsg: errorMessage }
		});

		mockPageStore.mockSetSubscribeValue({
			data: {
				orderName: 'order-one',
				deployment: {
					key: 'test-deployment',
					name: 'Deployment',
					description: 'desc'
				},
				orderDetail: { name: 'Order', description: 'desc' },
				rainlang: mockRainlang
			},
			url: new URL('http://localhost:3000/deploy')
		});

		render(DeployPage);

		await waitFor(() => {
			expect(screen.getByText(errorMessage)).toBeInTheDocument();
		});
	});
});
