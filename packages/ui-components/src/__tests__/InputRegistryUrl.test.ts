import { render, screen, fireEvent } from '@testing-library/svelte';
import { vi } from 'vitest';
import InputRegistryUrl from '../lib/components/input/InputRegistryUrl.svelte';
import userEvent from '@testing-library/user-event';
import { loadRegistryUrl } from '../lib/services/loadRegistryUrl';
import { initialRegistry } from '../__fixtures__/RegistryManager';
import { useRegistry } from '$lib/providers/registry/useRegistry';
import type { RegistryManager } from '$lib/providers/registry/RegistryManager';

vi.mock('../lib/services/loadRegistryUrl', () => ({
	loadRegistryUrl: vi.fn()
}));

vi.mock('../lib/providers/registry/useRegistry', () => ({
	useRegistry: vi.fn()
}));

describe('InputRegistryUrl', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.mocked(loadRegistryUrl).mockResolvedValue(undefined);
		vi.mocked(useRegistry).mockReturnValue(initialRegistry as RegistryManager);
	});

	it('should render input and button', () => {
		render(InputRegistryUrl);

		const input = screen.getByPlaceholderText('Enter URL to raw order registry file');
		const button = screen.getByText('Load registry URL');

		expect(input).toBeInTheDocument();
		expect(button).toBeInTheDocument();
	});

	it('should call loadRegistryUrl when button is clicked', async () => {
		render(InputRegistryUrl);

		const input = screen.getByPlaceholderText('Enter URL to raw order registry file');
		const testUrl = 'https://example.com/registry.json';
		await userEvent.clear(input);
		await userEvent.type(input, testUrl);

		const button = screen.getByText('Load registry URL');
		await fireEvent.click(button);

		expect(loadRegistryUrl).toHaveBeenCalledWith(testUrl, initialRegistry);
	});

	it('should load initial value from registry manager', () => {
		const initialUrl = 'https://example.com/registry.json';
		initialRegistry.getCurrentRegistry = vi.fn().mockReturnValue(initialUrl);
		render(InputRegistryUrl);

		const input = screen.getByPlaceholderText('Enter URL to raw order registry file');
		expect(input).toHaveValue(initialUrl);
	});

	it('should display error message when loadRegistryUrl fails', async () => {
		vi.mocked(loadRegistryUrl).mockRejectedValueOnce(new Error('Test error'));

		render(InputRegistryUrl);

		const button = screen.getByText('Load registry URL');
		await fireEvent.click(button);

		expect(await screen.findByTestId('registry-error')).toHaveTextContent('Test error');
	});

	it('should show loading state when request is in progress', async () => {
		vi.useFakeTimers();

		vi.mocked(loadRegistryUrl).mockImplementation(() => {
			return new Promise<void>((resolve) => {
				setTimeout(() => resolve(), 1000);
			});
		});

		const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });

		render(InputRegistryUrl);

		const button = screen.getByText('Load registry URL');
		await user.click(button);

		expect(screen.getByText('Loading registry...')).toBeInTheDocument();
		expect(button).toBeDisabled();

		await vi.runAllTimersAsync();

		expect(screen.getByText('Load registry URL')).toBeInTheDocument();
		expect(button).not.toBeDisabled();

		vi.useRealTimers();
	});
});
