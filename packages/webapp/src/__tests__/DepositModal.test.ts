import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import DepositModal from '$lib/components/DepositModal.svelte';
import type { ComponentProps } from 'svelte';
import type { Hex } from 'viem';
import truncateEthAddress from 'truncate-eth-address';
import { Float, type RaindexVault } from '@rainlanguage/raindex';

type ModalProps = ComponentProps<DepositModal>;

const { mockAppKitModalStore, mockConnectedStore } = await vi.hoisted(
	() => import('../lib/__mocks__/stores')
);

vi.mock('../lib/stores/wagmi', () => ({
	appKitModal: mockAppKitModalStore,
	connected: mockConnectedStore
}));

describe('DepositModal', () => {
	const mockVault = {
		token: {
			address: '0x123',
			symbol: 'TEST',
			decimals: '18'
		},
		getOwnerBalance: vi.fn().mockResolvedValue({
			value: {
				balance: Float.parse('1').value, // 1 token
				formattedBalance: '1'
			}
		}),
		chainId: 1,
		raindex: '0x123',
		balance: Float.parse('1').value as Float
	} as unknown as RaindexVault;

	const mockOnSubmit = vi.fn();

	const defaultProps = {
		open: true,
		onSubmit: mockOnSubmit,
		args: {
			vault: mockVault,
			account: '0x0000000000000000000000000000000000000000'
		}
	} as unknown as ModalProps;

	beforeEach(() => {
		vi.clearAllMocks();
		mockOnSubmit.mockClear();
	});

	it('renders deposit modal correctly', () => {
		render(DepositModal, defaultProps);
		expect(screen.getByTestId('modal-title')).toHaveTextContent('Deposit');
		expect(screen.getByTestId('deposit-button')).toBeInTheDocument();
	});

	it('handles deposit transaction correctly', async () => {
		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const depositButton = screen.getByTestId('deposit-button');
		await fireEvent.click(depositButton);

		expect(mockOnSubmit).toHaveBeenCalledOnce();
		const actualArg = mockOnSubmit.mock.calls[0][0];
		expect(actualArg.format().value).toBe('1');
	});

	it('shows error when amount exceeds balance', async () => {
		(mockVault.getOwnerBalance as Mock).mockResolvedValue({
			value: {
				balance: Float.parse('0.5').value, // 0.5 tokens
				formattedBalance: '0.5'
			}
		});
		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		expect(screen.getByTestId('amount-error')).toHaveTextContent(
			'Amount cannot exceed available balance.'
		);
	});

	it('disables continue button when amount is 0', async () => {
		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '0' } });

		const continueButton = screen.getByTestId('deposit-button');
		expect(continueButton).toBeDisabled();
	});

	it('disables continue button when amount exceeds balance', async () => {
		(mockVault.getOwnerBalance as Mock).mockResolvedValue({
			value: {
				balance: Float.parse('0.5').value, // 0.5 tokens
				formattedBalance: '0.5'
			}
		});
		render(DepositModal, defaultProps);

		// Wait for balance to be displayed
		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const continueButton = screen.getByTestId('deposit-button');
		expect(continueButton).toBeDisabled();
	});

	it('handles zero user balance correctly', async () => {
		(mockVault.getOwnerBalance as Mock).mockResolvedValue({
			value: {
				balance: Float.parse('0').value,
				formattedBalance: '0'
			}
		});

		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
			expect(screen.getByText(/0 TEST|0.0 TEST/)).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '0.1' } });

		const depositButton = screen.getByTestId('deposit-button');
		expect(depositButton).toBeDisabled();
		expect(screen.getByTestId('amount-error')).toHaveTextContent(
			'Amount cannot exceed available balance.'
		);
	});

	it('displays user balance correctly', async () => {
		(mockVault.getOwnerBalance as Mock).mockResolvedValue({
			value: {
				balance: Float.parse('2.5').value,
				formattedBalance: '2.5'
			}
		});

		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
			expect(screen.getByText('2.5 TEST')).toBeInTheDocument();
		});
	});

	it('shows error message when getUserBalance fails', async () => {
		vi.mocked(mockVault.getOwnerBalance).mockResolvedValue({
			value: undefined,
			error: {
				msg: 'Failed to get user balance.',
				readableMsg: 'Failed to get user balance.'
			}
		});

		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Failed to get user balance.')).toBeInTheDocument();
		});
	});

	it('shows wallet address in truncated form', async () => {
		(mockVault.getOwnerBalance as Mock).mockResolvedValue({
			value: {
				balance: Float.parse('1').value, // 1 token
				formattedBalance: '1'
			}
		});
		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(
				screen.getByText(truncateEthAddress('0x0000000000000000000000000000000000000000'))
			).toBeInTheDocument();
		});
	});

	it('shows the cancel button that closes the modal', async () => {
		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Cancel')).toBeInTheDocument();
		});

		const cancelButton = screen.getByText('Cancel');
		await fireEvent.click(cancelButton);

		await waitFor(() => {
			expect(screen.queryByTestId('modal-title')).not.toBeInTheDocument();
		});
	});

	it('shows the WalletConnect button when account is not provided', async () => {
		render(DepositModal, {
			...defaultProps,
			args: {
				...defaultProps.args,
				account: null as unknown as Hex
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Connect your wallet to continue.')).toBeInTheDocument();
			expect(screen.queryByTestId('deposit-button')).not.toBeInTheDocument();
		});
	});
});
