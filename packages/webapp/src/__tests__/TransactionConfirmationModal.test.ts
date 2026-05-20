import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import TransactionConfirmationModal from '$lib/components/TransactionConfirmationModal.svelte';
import type { TransactionConfirmationProps } from '@rainlanguage/ui-components';
import type { RaindexOrder } from '@rainlanguage/raindex';
import { handleWalletConfirmation } from '$lib/services/handleWalletConfirmation';

vi.mock('$lib/services/handleWalletConfirmation', () => ({
	handleWalletConfirmation: vi.fn()
}));

describe('TransactionConfirmationModal', () => {
	const mockCalldata = '0x1234567890abcdef';
	const mockTxHash = '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890';

	const mockOrder = {
		id: '0x1',
		orderBytes: '0x2',
		orderHash: '0x3',
		owner: '0x4',
		outputs: [],
		inputs: [],
		raindex: 0x5,
		active: true,
		timestampAdded: BigInt(1234567890),
		tradesCount: 0
	} as unknown as RaindexOrder;

	const testModalTitle = 'Test Modal Title';
	const defaultProps: TransactionConfirmationProps = {
		open: true,
		modalTitle: testModalTitle,
		args: {
			chainId: 1,
			toAddress: '0x5',
			calldata: mockCalldata,
			onConfirm: vi.fn(),
			entity: mockOrder
		}
	};

	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetAllMocks();
		vi.useFakeTimers();
		vi.mocked(handleWalletConfirmation).mockResolvedValue({
			state: { status: 'confirmed' },
			hash: mockTxHash
		});
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	it('shows awaiting confirmation state initially and correct title', () => {
		render(TransactionConfirmationModal, defaultProps);

		expect(screen.getByText(defaultProps.modalTitle as string)).toBeInTheDocument();
		expect(screen.getByText(testModalTitle)).toBeInTheDocument();
		expect(screen.getByText('Please confirm this transaction in your wallet.')).toBeInTheDocument();
		expect(screen.getByTestId('transaction-modal')).toBeInTheDocument();
	});

	it('handles successful transaction flow', async () => {
		render(TransactionConfirmationModal, defaultProps);

		await waitFor(() => {
			expect(handleWalletConfirmation).toHaveBeenCalledWith(defaultProps.args);
			expect(screen.getByText('Transaction submitted')).toBeInTheDocument();
			expect(
				screen.getByText('Transaction has been submitted to the network.')
			).toBeInTheDocument();
			expect(screen.getByText('✅')).toBeInTheDocument();
			expect(screen.queryByText('Dismiss')).toBeInTheDocument();
		});
	});

	it('handles chain switch error', async () => {
		const errorMessage = 'Failed to switch chain';
		vi.mocked(handleWalletConfirmation).mockResolvedValue({
			state: {
				status: 'error',
				reason: errorMessage
			}
		});

		render(TransactionConfirmationModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Confirmation failed')).toBeInTheDocument();
			expect(screen.getByText(errorMessage)).toBeInTheDocument();
			expect(screen.getByText('Dismiss')).toBeInTheDocument();
			expect(screen.getByText('❌')).toBeInTheDocument();
		});
	});

	it('handles transaction rejection', async () => {
		vi.mocked(handleWalletConfirmation).mockResolvedValue({
			state: {
				status: 'rejected',
				reason: 'User rejected transaction'
			}
		});

		render(TransactionConfirmationModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Confirmation failed')).toBeInTheDocument();
			expect(screen.getByText('User rejected transaction')).toBeInTheDocument();
			expect(screen.getByText('Dismiss')).toBeInTheDocument();
			expect(screen.getByText('❌')).toBeInTheDocument();
		});
	});

	it('handles non-Error chain switch failure', async () => {
		vi.mocked(handleWalletConfirmation).mockResolvedValue({
			state: {
				status: 'error',
				reason: 'Failed to switch chain'
			}
		});

		render(TransactionConfirmationModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Confirmation failed')).toBeInTheDocument();
			expect(screen.getByText('Failed to switch chain')).toBeInTheDocument();
			expect(screen.getByText('Dismiss')).toBeInTheDocument();
			expect(screen.getByText('❌')).toBeInTheDocument();
		});
	});

	it('auto-closes modal after 2 seconds when transaction is confirmed', async () => {
		const setTimeoutSpy = vi.spyOn(global, 'setTimeout');

		render(TransactionConfirmationModal, defaultProps);

		// Wait for the transaction to be confirmed
		await waitFor(() => {
			expect(screen.getByText('Transaction submitted')).toBeInTheDocument();
		});

		// Verify that setTimeout was called with 2000ms delay
		expect(setTimeoutSpy).toHaveBeenCalledWith(expect.any(Function), 2000);

		setTimeoutSpy.mockRestore();
	});

	it('clears timeout when modal is manually dismissed', async () => {
		const clearTimeoutSpy = vi.spyOn(global, 'clearTimeout');

		render(TransactionConfirmationModal, defaultProps);

		// Wait for the transaction to be confirmed
		await waitFor(() => {
			expect(screen.getByText('Transaction submitted')).toBeInTheDocument();
		});

		// Click dismiss button
		const dismissButton = screen.getByText('Dismiss');
		dismissButton.click();

		// Verify that clearTimeout was called
		expect(clearTimeoutSpy).toHaveBeenCalled();

		clearTimeoutSpy.mockRestore();
	});
});
