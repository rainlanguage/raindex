import { render, screen, fireEvent } from '@testing-library/svelte';
import { vi } from 'vitest';
import InputRainlangUrl from '../lib/components/input/InputRainlangUrl.svelte';
import userEvent from '@testing-library/user-event';
import { loadRainlangUrl } from '../lib/services/loadRainlangUrl';
import { initialRainlang } from '../__fixtures__/RainlangManager';
import { useRainlang } from '$lib/providers/rainlang/useRainlang';
import type { RainlangManager } from '$lib/providers/rainlang/RainlangManager';

vi.mock('../lib/services/loadRainlangUrl', () => ({
	loadRainlangUrl: vi.fn()
}));

vi.mock('../lib/providers/rainlang/useRainlang', () => ({
	useRainlang: vi.fn()
}));

describe('InputRainlangUrl', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.mocked(loadRainlangUrl).mockResolvedValue(undefined);
		vi.mocked(useRainlang).mockReturnValue(initialRainlang as RainlangManager);
	});

	it('should render input and button', () => {
		render(InputRainlangUrl);

		const input = screen.getByPlaceholderText('Enter URL to raw order rainlang file');
		const button = screen.getByText('Load rainlang URL');

		expect(input).toBeInTheDocument();
		expect(button).toBeInTheDocument();
	});

	it('should call loadRainlangUrl when button is clicked', async () => {
		render(InputRainlangUrl);

		const input = screen.getByPlaceholderText('Enter URL to raw order rainlang file');
		const testUrl = 'https://example.com/rainlang.json';
		await userEvent.clear(input);
		await userEvent.type(input, testUrl);

		const button = screen.getByText('Load rainlang URL');
		await fireEvent.click(button);

		expect(loadRainlangUrl).toHaveBeenCalledWith(testUrl, initialRainlang);
	});

	it('should load initial value from rainlang manager', () => {
		const initialUrl = 'https://example.com/rainlang.json';
		initialRainlang.getCurrentRainlang = vi.fn().mockReturnValue(initialUrl);
		render(InputRainlangUrl);

		const input = screen.getByPlaceholderText('Enter URL to raw order rainlang file');
		expect(input).toHaveValue(initialUrl);
	});

	it('should display error message when loadRainlangUrl fails', async () => {
		vi.mocked(loadRainlangUrl).mockRejectedValueOnce(new Error('Test error'));

		render(InputRainlangUrl);

		const button = screen.getByText('Load rainlang URL');
		await fireEvent.click(button);

		expect(await screen.findByTestId('rainlang-error')).toHaveTextContent('Test error');
	});

	it('should show loading state when request is in progress', async () => {
		vi.useFakeTimers();

		vi.mocked(loadRainlangUrl).mockImplementation(() => {
			return new Promise<void>((resolve) => {
				setTimeout(() => resolve(), 1000);
			});
		});

		const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });

		render(InputRainlangUrl);

		const button = screen.getByText('Load rainlang URL');
		await user.click(button);

		expect(screen.getByText('Loading rainlang...')).toBeInTheDocument();
		expect(button).toBeDisabled();

		await vi.runAllTimersAsync();

		expect(screen.getByText('Load rainlang URL')).toBeInTheDocument();
		expect(button).not.toBeDisabled();

		vi.useRealTimers();
	});
});
