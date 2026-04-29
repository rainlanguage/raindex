import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { QueryClient } from '@tanstack/svelte-query';
import VaultDetail from '../lib/components/detail/VaultDetail.svelte';
import { readable, writable } from 'svelte/store';
import { darkChartTheme } from '../lib/utils/lightweightChartsThemes';
import userEvent from '@testing-library/user-event';
import type { ComponentProps } from 'svelte';
import { useRaindexClient } from '$lib/hooks/useRaindexClient';
import { RaindexClient, RaindexVault, type RaindexOrderAsIO } from '@rainlanguage/raindex';
import { useAccount } from '../lib/providers/wallet/useAccount';
import { QKEY_VAULT } from '$lib/queries/keys';
import { useToasts } from '../lib/providers/toasts/useToasts';
import { invalidateTanstackQueries } from '$lib/queries/queryClient';
import { getExplorerLink } from '$lib/services/getExplorerLink';

type VaultDetailProps = ComponentProps<VaultDetail>;

vi.mock('../lib/providers/wallet/useAccount', () => ({
	useAccount: vi.fn()
}));

vi.mock('$lib/hooks/useRaindexClient', () => ({
	useRaindexClient: vi.fn()
}));

vi.mock('@rainlanguage/raindex', () => ({
	RaindexClient: vi.fn()
}));

vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

vi.mock('$lib/services/modal', () => ({
	handleDepositModal: vi.fn(),
	handleWithdrawModal: vi.fn()
}));

vi.mock('$lib/queries/queryClient', () => ({
	invalidateTanstackQueries: vi.fn()
}));

vi.mock('$lib/providers/toasts/useToasts', () => ({
	useToasts: vi.fn()
}));

vi.mock('$lib/services/getExplorerLink', () => ({
	getExplorerLink: vi.fn()
}));

const mockErrToast = vi.fn();

const defaultProps: VaultDetailProps = {
	chainId: 1,
	raindexAddress: '0x00',
	id: '100',
	lightweightChartsTheme: readable(darkChartTheme),
	onDeposit: vi.fn(),
	onWithdraw: vi.fn()
} as unknown as VaultDetailProps;

const mockMatchesAccount = vi.fn();

describe('VaultDetail', () => {
	let queryClient: QueryClient;
	let mockRaindexClient: RaindexClient;
	let mockData: RaindexVault;

	beforeEach(async () => {
		vi.clearAllMocks();
		queryClient = new QueryClient();

		mockMatchesAccount.mockReturnValue(true);

		(useAccount as Mock).mockReturnValue({
			matchesAccount: mockMatchesAccount
		});

		(useToasts as Mock).mockReturnValue({
			errToast: mockErrToast,
			toasts: writable([]),
			removeToast: vi.fn()
		});

		(getExplorerLink as Mock).mockReturnValue(
			'https://etherscan.io/address/0x1234567890123456789012345678901234567890'
		);

		mockRaindexClient = {
			getVault: vi.fn()
		} as unknown as RaindexClient;
		(useRaindexClient as Mock).mockReturnValue(mockRaindexClient);

		mockData = {
			id: '1',
			chainId: 1,
			owner: '0x1234567890123456789012345678901234567890',
			vaultId: BigInt(1000),
			token: {
				id: '0x456',
				address: '0x456',
				name: 'USDC coin',
				symbol: 'USDC',
				decimals: '6'
			},
			balance: BigInt(100000000000),
			formattedBalance: '100000',
			ordersAsInput: [],
			ordersAsOutput: [],
			raindex: '0x00'
		} as unknown as RaindexVault;
		(mockRaindexClient.getVault as Mock).mockResolvedValue({ value: mockData });
	});

	it('calls the vault detail query fn with the correct vault id', async () => {
		render(VaultDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		expect(mockRaindexClient.getVault).toHaveBeenCalledWith(1, '0x00', '100');
	});

	it('shows the correct empty message when the query returns no data', async () => {
		(mockRaindexClient.getVault as Mock).mockResolvedValue({ value: null });

		render(VaultDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			expect(screen.getByText('Vault not found')).toBeInTheDocument();
		});
	});

	it('shows the correct data when the query returns data', async () => {
		render(VaultDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			expect(screen.getByTestId('vaultDetailTokenName')).toHaveTextContent('USDC coin');
			expect(screen.getByTestId('vaultDetailVaultId')).toHaveTextContent('Vault ID 0x3e8');
			expect(screen.getByTestId('vaultDetailOwnerAddress')).toHaveTextContent(
				'Owner address 0x123'
			);
			expect(screen.getByTestId('vaultDetailTokenAddress')).toHaveTextContent(
				'Token address 0x456'
			);
			expect(screen.getByTestId('vaultDetailBalance')).toHaveTextContent('Balance 100000 USDC');
			expect(screen.queryByTestId('vaultDetailOrdersAsInput')).toHaveTextContent('None');
			expect(screen.queryByTestId('vaultDetailOrdersAsOutput')).toHaveTextContent('None');
		});
	});

	it('shows deposit/withdraw buttons when account matches are met', async () => {
		mockMatchesAccount.mockReturnValue(true);

		render(VaultDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			expect(screen.getAllByTestId('deposit-button')).toHaveLength(1);
			expect(screen.getAllByTestId('withdraw-button')).toHaveLength(1);
		});
	});

	it("doesn't show deposit/withdraw buttons when account doesn't match", async () => {
		mockMatchesAccount.mockReturnValue(false);

		render(VaultDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			expect(screen.queryByTestId('depositOrWithdrawButton')).not.toBeInTheDocument();
		});
	});

	it('refresh button triggers query invalidation when clicked', async () => {
		// @ts-expect-error - we are mutating the mock data
		mockData.ordersAsInput = [{ id: '1' }] as unknown as RaindexOrderAsIO[];
		// @ts-expect-error - we are mutating the mock data
		mockData.ordersAsOutput = [{ id: '2' }] as unknown as RaindexOrderAsIO[];

		render(VaultDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(async () => {
			const refreshButton = await screen.getByTestId('top-refresh');
			await userEvent.click(refreshButton);
			expect(invalidateTanstackQueries).toHaveBeenCalledWith(queryClient, [
				'100',
				QKEY_VAULT + '100'
			]);
		});
	});

	it('failed query invalidation triggers a toast', async () => {
		// @ts-expect-error - we are mutating the mock data
		mockData.ordersAsInput = [{ id: '1' }] as unknown as RaindexOrderAsIO[];
		// @ts-expect-error - we are mutating the mock data
		mockData.ordersAsOutput = [{ id: '2' }] as unknown as RaindexOrderAsIO[];

		(invalidateTanstackQueries as Mock).mockRejectedValue(new Error('Failed to refresh'));

		render(VaultDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(async () => {
			const refreshButton = screen.getByTestId('top-refresh');
			await userEvent.click(refreshButton);
			expect(mockErrToast).toHaveBeenCalledWith('Failed to refresh');
		});
	});

	it('renders owner address as explorer link when explorer is available', async () => {
		const explorerUrl = 'https://etherscan.io/address/0x1234567890123456789012345678901234567890';
		(getExplorerLink as Mock).mockReturnValue(explorerUrl);

		render(VaultDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			const ownerLink = screen.getByRole('link', {
				name: /0x1234567890123456789012345678901234567890/i
			});
			expect(ownerLink).toBeInTheDocument();
			expect(ownerLink).toHaveAttribute('href', explorerUrl);
			expect(ownerLink).toHaveAttribute('target', '_blank');
			expect(ownerLink).toHaveAttribute('rel', 'noopener noreferrer');
		});
	});

	it('falls back to Hash component when no explorer link is available', async () => {
		(getExplorerLink as Mock).mockReturnValue('');

		render(VaultDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			expect(screen.getByTestId('vaultDetailOwnerAddress')).toHaveTextContent('0x123');
			expect(
				screen.queryByRole('link', { name: /0x1234567890123456789012345678901234567890/i })
			).not.toBeInTheDocument();
		});
	});
});
