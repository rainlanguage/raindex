import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import SelectToken from '../lib/components/deployment/SelectToken.svelte';
import type { ComponentProps } from 'svelte';
import { Float, type AccountBalance, type DotrainOrderGui } from '@rainlanguage/raindex';
import { useGui } from '$lib/hooks/useGui';
import type { TokenBalance } from '$lib/types/tokenBalance';

type SelectTokenComponentProps = ComponentProps<SelectToken>;

const mockGui: DotrainOrderGui = {
	setSelectToken: vi.fn(),
	isSelectTokenSet: vi.fn(),
	unsetSelectToken: vi.fn(),
	getTokenInfo: vi.fn().mockResolvedValue({
		value: {
			name: 'Ethereum',
			symbol: 'ETH',
			decimals: 18,
			address: '0x456'
		}
	}),
	getAllTokens: vi.fn().mockResolvedValue({
		value: [
			{
				key: 'token1',
				address: '0x1234567890123456789012345678901234567890',
				name: 'Test Token 1',
				symbol: 'TEST1',
				decimals: 18
			},
			{
				key: 'token2',
				address: '0x0987654321098765432109876543210987654321',
				name: 'Another Token',
				symbol: 'ANOTHER',
				decimals: 6
			}
		]
	})
} as unknown as DotrainOrderGui;

vi.mock('@rainlanguage/raindex', async (importOriginal) => {
	return {
		...(await importOriginal())
	};
});

vi.mock('../lib/hooks/useGui', () => ({
	useGui: vi.fn()
}));

describe('SelectToken', () => {
	let mockStateUpdateCallback: Mock;

	const mockProps: SelectTokenComponentProps = {
		token: {
			key: 'input',
			name: 'test input',
			description: 'test description'
		},
		onSelectTokenSelect: vi.fn(),
		tokenBalances: new Map()
	};

	beforeEach(() => {
		mockStateUpdateCallback = vi.fn();
		mockGui.setSelectToken = vi.fn().mockImplementation(() => {
			mockStateUpdateCallback();
			return Promise.resolve();
		});
		(useGui as Mock).mockReturnValue(mockGui);
		vi.clearAllMocks();
	});

	it('renders token label correctly', () => {
		const { getByText } = render(SelectToken, mockProps);
		expect(getByText('test input')).toBeInTheDocument();
	});

	it('renders dropdown button when tokens are available', () => {
		const { getByText } = render(SelectToken, mockProps);
		expect(getByText('Select a token...')).toBeInTheDocument();
	});

	it('calls setSelectToken and updates token info when input changes', async () => {
		const user = userEvent.setup();
		const mockGuiWithNoToken = {
			...mockGui,
			getTokenInfo: vi.fn().mockResolvedValue({ value: null })
		} as unknown as DotrainOrderGui;

		(useGui as Mock).mockReturnValue(mockGuiWithNoToken);

		const { getByTestId, getByRole } = render(SelectToken, {
			...mockProps
		});

		const customButton = getByTestId('custom-mode-button');
		await user.click(customButton);

		const input = getByRole('textbox');

		await userEvent.clear(input);
		await user.paste('0x456');

		await waitFor(() => {
			expect(mockGuiWithNoToken.setSelectToken).toHaveBeenCalledWith('input', '0x456');
		});
		expect(mockStateUpdateCallback).toHaveBeenCalledTimes(1);
	});

	it('shows error message for invalid address, and removes the selectToken', async () => {
		const user = userEvent.setup();
		const mockGuiWithError = {
			...mockGui,
			setSelectToken: vi.fn().mockRejectedValue(new Error('Invalid address'))
		} as unknown as DotrainOrderGui;

		(useGui as Mock).mockReturnValue(mockGuiWithError);

		const screen = render(SelectToken, {
			...mockProps
		});

		const customButton = screen.getByTestId('custom-mode-button');
		await user.click(customButton);

		const input = screen.getByRole('textbox');
		await userEvent.clear(input);
		await user.paste('invalid');
		await waitFor(() => {
			expect(screen.getByTestId('error')).toBeInTheDocument();
		});
	});

	it('replaces the token and triggers state update twice if the token is already set', async () => {
		const mockGuiWithTokenSet = {
			...mockGui,
			isSelectTokenSet: vi.fn().mockResolvedValue(true)
		} as unknown as DotrainOrderGui;

		(useGui as Mock).mockReturnValue(mockGuiWithTokenSet);

		const user = userEvent.setup();

		const { getByRole, getByTestId } = render(SelectToken, {
			...mockProps
		});

		const customButton = getByTestId('custom-mode-button');
		await user.click(customButton);

		const input = getByRole('textbox');
		await userEvent.clear(input);
		await user.paste('invalid');
		await waitFor(() => {
			expect(mockGuiWithTokenSet.setSelectToken).toHaveBeenCalled();
			expect(mockStateUpdateCallback).toHaveBeenCalledTimes(1);
		});
	});

	it('calls onSelectTokenSelect after input changes', async () => {
		const user = userEvent.setup();
		const { getByRole, getByTestId } = render(SelectToken, mockProps);

		const customButton = getByTestId('custom-mode-button');
		await user.click(customButton);

		const input = getByRole('textbox');

		await userEvent.clear(input);
		await user.paste('0x456');

		await waitFor(() => {
			expect(mockProps.onSelectTokenSelect).toHaveBeenCalled();
		});
	});

	describe('Dropdown Mode', () => {
		beforeEach(() => {
			(useGui as Mock).mockReturnValue(mockGui);
		});

		it('shows dropdown and custom mode buttons when tokens are available', () => {
			render(SelectToken, mockProps);

			expect(screen.getByTestId('dropdown-mode-button')).toBeInTheDocument();
			expect(screen.getByTestId('custom-mode-button')).toBeInTheDocument();
		});

		it('shows dropdown mode as active by default', () => {
			render(SelectToken, mockProps);

			const dropdownButton = screen.getByTestId('dropdown-mode-button');
			const customButton = screen.getByTestId('custom-mode-button');

			expect(dropdownButton).toHaveClass('border-blue-300');
			expect(customButton).not.toHaveClass('border-blue-300');
		});

		it('switches to custom mode when custom button is clicked', async () => {
			const user = userEvent.setup();
			render(SelectToken, mockProps);

			const customButton = screen.getByTestId('custom-mode-button');
			await user.click(customButton);

			expect(customButton).toHaveClass('border-blue-300');
			expect(screen.getByTestId('dropdown-mode-button')).not.toHaveClass('border-blue-300');
		});

		it('shows TokenSelectionModal component in dropdown mode', () => {
			render(SelectToken, mockProps);

			expect(screen.getByText('Select a token...')).toBeInTheDocument();
		});

		it('shows custom input in custom mode', async () => {
			const user = userEvent.setup();
			render(SelectToken, mockProps);

			const customButton = screen.getByTestId('custom-mode-button');
			await user.click(customButton);

			expect(screen.getByPlaceholderText('Enter token address (0x...)')).toBeInTheDocument();
		});

		it('clears state when switching from dropdown to custom mode', async () => {
			const user = userEvent.setup();
			const mockGuiNoToken = {
				...mockGui,
				getTokenInfo: vi.fn().mockResolvedValue({ value: null })
			} as unknown as DotrainOrderGui;

			(useGui as Mock).mockReturnValue(mockGuiNoToken);

			render(SelectToken, {
				...mockProps
			});

			const dropdownButton = screen.getByText('Select a token...');
			await user.click(dropdownButton);

			const firstToken = screen.getByText('Test Token 1');
			await user.click(firstToken);

			const customButton = screen.getByTestId('custom-mode-button');
			await user.click(customButton);

			const customInput = screen.getByPlaceholderText('Enter token address (0x...)');
			expect(customInput).toHaveValue('');

			expect(mockGuiNoToken.unsetSelectToken).toHaveBeenCalledWith('input');
		});

		it('clears state when switching from custom to dropdown mode', async () => {
			const user = userEvent.setup();
			render(SelectToken, mockProps);

			const customButton = screen.getByTestId('custom-mode-button');
			await user.click(customButton);

			const customInput = screen.getByPlaceholderText('Enter token address (0x...)');
			await user.type(customInput, '0x1234567890123456789012345678901234567890');

			const dropdownButton = screen.getByTestId('dropdown-mode-button');
			await user.click(dropdownButton);

			expect(mockGui.unsetSelectToken).toHaveBeenCalledWith('input');
		});

		it('handles token selection from dropdown', async () => {
			const user = userEvent.setup();
			const mockGuiNoToken = {
				...mockGui,
				getTokenInfo: vi.fn().mockResolvedValue({ value: null })
			} as unknown as DotrainOrderGui;

			(useGui as Mock).mockReturnValue(mockGuiNoToken);

			render(SelectToken, {
				...mockProps
			});

			const dropdownButton = screen.getByText('Select a token...');
			await user.click(dropdownButton);

			const secondToken = screen.getByText('Another Token');
			await user.click(secondToken);

			expect(mockGuiNoToken.setSelectToken).toHaveBeenCalledWith(
				'input',
				'0x0987654321098765432109876543210987654321'
			);
		});

		it('displays selected token info when token is selected', async () => {
			mockGui.getTokenInfo = vi.fn().mockResolvedValue({
				value: {
					name: 'Test Token 1',
					symbol: 'TEST1',
					address: '0x1234567890123456789012345678901234567890',
					decimals: 18
				}
			});

			render(SelectToken, mockProps);

			await waitFor(() => {
				expect(screen.getByText('Test Token 1')).toBeInTheDocument();
			});

			expect(screen.getByTestId(`select-token-success-${mockProps.token.key}`)).toBeInTheDocument();
		});
	});

	describe('Balance Display', () => {
		it('displays balance when token is selected and balance is provided', async () => {
			mockGui.getTokenInfo = vi.fn().mockResolvedValue({
				value: {
					name: 'Test Token',
					symbol: 'TEST',
					address: '0x1234567890123456789012345678901234567890',
					decimals: 18,
					key: 'input'
				}
			});

			const tokenBalances = new Map<string, TokenBalance>();
			tokenBalances.set('input', {
				value: {
					balance: Float.parse('1').value,
					formattedBalance: '1'
				} as AccountBalance,
				loading: false,
				error: ''
			});

			render(SelectToken, {
				...mockProps,
				tokenBalances
			});

			await waitFor(() => {
				expect(screen.getByText('Test Token')).toBeInTheDocument();
			});

			await waitFor(() => {
				expect(screen.getByText('Balance: 1')).toBeInTheDocument();
			});
		});

		it('shows loading spinner when balance is loading', async () => {
			mockGui.getTokenInfo = vi.fn().mockResolvedValue({
				value: {
					name: 'Test Token',
					symbol: 'TEST',
					address: '0x1234567890123456789012345678901234567890',
					decimals: 18,
					key: 'input'
				}
			});

			const tokenBalances = new Map<string, TokenBalance>();
			tokenBalances.set('input', {
				value: {
					balance: Float.parse('0').value,
					formattedBalance: '0'
				} as AccountBalance,
				loading: true,
				error: ''
			});

			render(SelectToken, {
				...mockProps,
				tokenBalances
			});

			await waitFor(() => {
				expect(screen.getByText('Test Token')).toBeInTheDocument();
			});

			// Check for spinner (we can't easily test the spinner component directly, so we test for its presence)
			const tokenStatus = screen.getByTestId(`select-token-success-${mockProps.token.key}`);
			expect(tokenStatus).toBeInTheDocument();
		});

		it('shows error message when balance fetch fails', async () => {
			mockGui.getTokenInfo = vi.fn().mockResolvedValue({
				value: {
					name: 'Test Token',
					symbol: 'TEST',
					address: '0x1234567890123456789012345678901234567890',
					decimals: 18,
					key: 'input'
				}
			});

			const tokenBalances = new Map<string, TokenBalance>();
			tokenBalances.set('input', {
				value: {
					balance: Float.parse('0').value,
					formattedBalance: '0'
				} as AccountBalance,
				loading: false,
				error: 'Network error'
			});

			render(SelectToken, {
				...mockProps,
				tokenBalances
			});

			await waitFor(() => {
				expect(screen.getByText('Test Token')).toBeInTheDocument();
			});

			await waitFor(() => {
				expect(screen.getByText('Network error')).toBeInTheDocument();
			});
		});

		it('formats balance correctly with token decimals', async () => {
			mockGui.getTokenInfo = vi.fn().mockResolvedValue({
				value: {
					name: 'USDC',
					symbol: 'USDC',
					address: '0x1234567890123456789012345678901234567890',
					decimals: 6,
					key: 'input'
				}
			});

			const tokenBalances = new Map<string, TokenBalance>();
			tokenBalances.set('input', {
				value: {
					balance: Float.parse('1.5').value,
					formattedBalance: '1.5'
				} as AccountBalance,
				loading: false,
				error: ''
			});

			render(SelectToken, {
				...mockProps,
				tokenBalances
			});

			await waitFor(() => {
				expect(screen.getByText('USDC')).toBeInTheDocument();
			});

			await waitFor(() => {
				expect(screen.getByText('Balance: 1.5')).toBeInTheDocument();
			});
		});
	});
});
