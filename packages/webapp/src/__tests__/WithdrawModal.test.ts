import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import WithdrawModal from '$lib/components/WithdrawModal.svelte';
import type { ComponentProps } from 'svelte';
import { Float, type RaindexVault } from '@rainlanguage/raindex';
import type { Hex } from 'viem';
import truncateEthAddress from 'truncate-eth-address';

type ModalProps = ComponentProps<WithdrawModal>;

const { mockAppKitModalStore, mockConnectedStore } = await vi.hoisted(
	() => import('../lib/__mocks__/stores')
);

vi.mock('../lib/stores/wagmi', () => ({
	appKitModal: mockAppKitModalStore,
	connected: mockConnectedStore
}));

describe('WithdrawModal', () => {
	const mockVault = {
		chainId: 1,
		raindex: '0x123',
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
		vaultId: '1',
		balance: Float.parse('1').value as Float,
		formattedBalance: '1'
	} as unknown as RaindexVault;

	const mockOnSubmit = vi.fn();

	const defaultProps: ModalProps = {
		open: true,
		args: {
			vault: mockVault,
			account: '0x0000000000000000000000000000000000000000' as Hex
		},
		onSubmit: mockOnSubmit
	};

	beforeEach(() => {
		vi.clearAllMocks();
		mockOnSubmit.mockClear();
	});

	it('renders withdraw modal correctly', () => {
		render(WithdrawModal, defaultProps);
		expect(screen.getByTestId('modal-title')).toHaveTextContent('Withdraw');
		expect(screen.getByTestId('withdraw-button')).toBeInTheDocument();
	});

	it('calls onSubmit with the amount when withdraw button is clicked', async () => {
		render(WithdrawModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const amountInput = screen.getByRole('textbox');
		await fireEvent.input(amountInput, { target: { value: '1' } });

		const withdrawButton = screen.getByTestId('withdraw-button');
		await fireEvent.click(withdrawButton);

		expect(defaultProps.onSubmit).toHaveBeenCalledTimes(1);
		const actualArg = mockOnSubmit.mock.calls[0][0];
		expect(actualArg.format().value).toBe('1');
	});

	it('shows error when amount exceeds balance', async () => {
		const mockVaultWithBalance = {
			...mockVault,
			balance: Float.parse('0.5').value as Float
		} as unknown as RaindexVault;

		render(WithdrawModal, {
			...defaultProps,
			args: {
				...defaultProps.args,
				vault: mockVaultWithBalance
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const amountInput = screen.getByRole('textbox');
		await fireEvent.input(amountInput, { target: { value: '1' } });

		expect(screen.getByTestId('amount-error')).toHaveTextContent(
			'Amount cannot exceed available balance.'
		);
	});

	it('disables continue button when amount is 0', async () => {
		render(WithdrawModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const amountInput = screen.getByRole('textbox');
		await fireEvent.input(amountInput, { target: { value: '0' } });

		const continueButton = screen.getByTestId('withdraw-button');
		expect(continueButton).toBeDisabled();
	});

	it('disables continue button when amount exceeds balance', async () => {
		const mockVaultWithBalance = {
			...mockVault,
			balance: Float.parse('0.5').value as Float
		} as unknown as RaindexVault;

		render(WithdrawModal, {
			...defaultProps,
			args: {
				...defaultProps.args,
				vault: mockVaultWithBalance
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const amountInput = screen.getByRole('textbox');
		await fireEvent.input(amountInput, { target: { value: '1' } });

		const continueButton = screen.getByTestId('withdraw-button');
		expect(continueButton).toBeDisabled();
	});

	it('handles zero vault balance correctly', async () => {
		const mockVaultWithZeroBalance = {
			...mockVault,
			balance: Float.parse('0').value as Float,
			formattedBalance: '0'
		} as unknown as RaindexVault;

		render(WithdrawModal, {
			...defaultProps,
			args: {
				...defaultProps.args,
				vault: mockVaultWithZeroBalance
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Balance of vault')).toBeInTheDocument();
			expect(screen.getByText('0 TEST')).toBeInTheDocument();
		});

		const withdrawInput = screen.getByRole('textbox');
		await fireEvent.input(withdrawInput, { target: { value: '0.1' } });

		const withdrawButton = screen.getByTestId('withdraw-button');
		expect(withdrawButton).toBeDisabled();
		expect(screen.getByTestId('amount-error')).toHaveTextContent(
			'Amount cannot exceed available balance.'
		);
	});

	it('displays vault balance correctly', async () => {
		const mockVaultWithBalance = {
			...mockVault,
			balance: Float.parse('3.7').value as Float,
			formattedBalance: '3.7'
		} as unknown as RaindexVault;

		render(WithdrawModal, {
			...defaultProps,
			args: {
				...defaultProps.args,
				vault: mockVaultWithBalance
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Balance of vault')).toBeInTheDocument();
			expect(screen.getByText('3.7 TEST')).toBeInTheDocument();
		});
	});

	it('shows wallet address in truncated form', async () => {
		render(WithdrawModal, defaultProps);

		await waitFor(() => {
			expect(
				screen.getByText(truncateEthAddress('0x0000000000000000000000000000000000000000'))
			).toBeInTheDocument();
		});
	});

	it('shows the cancel button that closes the modal', async () => {
		render(WithdrawModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Cancel')).toBeInTheDocument();
		});

		const cancelButton = screen.getByText('Cancel');
		await fireEvent.click(cancelButton);
	});

	it('shows the WalletConnect button when account is not provided', async () => {
		render(WithdrawModal, {
			...defaultProps,
			args: {
				...defaultProps.args,
				account: null as unknown as Hex
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Connect your wallet to continue.')).toBeInTheDocument();
			expect(screen.queryByTestId('withdraw-button')).not.toBeInTheDocument();
		});
	});
});
