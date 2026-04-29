import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi, describe } from 'vitest';
import { expect } from '../lib/test/matchers';
import { QueryClient } from '@tanstack/svelte-query';
import VaultBalanceChangesTable from '../lib/components/tables/VaultBalanceChangesTable.svelte';
import type {
	RaindexVault,
	RaindexVaultBalanceChange,
	RaindexVaultBalanceChangeType
} from '@rainlanguage/raindex';
import { formatTimestampSecondsAsLocal } from '../lib/services/time';
import { VAULT_BALANCE_CHANGE_LABELS } from '../lib/utils/vaultBalanceChangeLabels';

const TYPE_DISPLAY_NAMES: Record<RaindexVaultBalanceChangeType, string> = {
	deposit: 'Deposit',
	withdrawal: 'Withdrawal',
	takeOrder: 'Take order',
	clear: 'Clear',
	clearBounty: 'Clear Bounty',
	unknown: 'Unknown'
};

const createMockVaultBalanceChange = (
	overrides: Partial<RaindexVaultBalanceChange> = {}
): RaindexVaultBalanceChange => {
	const type = (overrides.type as RaindexVaultBalanceChangeType) || 'withdrawal';
	return {
		type,
		typeDisplayName: TYPE_DISPLAY_NAMES[type],
		amount: BigInt(1000),
		formattedAmount: '0.1',
		oldBalance: BigInt(5000),
		formattedOldBalance: '0.5',
		newBalance: BigInt(4000),
		formattedNewBalance: '0.4',
		timestamp: BigInt(1625247600),
		vaultId: BigInt(100),
		token: {
			id: 'token1',
			address: '0xTokenAddress1',
			name: 'Token1',
			symbol: 'TKN1',
			decimals: '4'
		},
		transaction: {
			id: 'tx1',
			from: '0xUser1',
			timestamp: BigInt(1625247600),
			blockNumber: BigInt(1234567890)
		},
		raindex: '0x00',
		...overrides
	} as unknown as RaindexVaultBalanceChange;
};

test('renders the vault list table with correct data', async () => {
	const queryClient = new QueryClient();

	const mockVaultBalanceChanges: RaindexVaultBalanceChange[] = [createMockVaultBalanceChange()];
	const mockVault: RaindexVault = {
		id: 'vault1',
		getBalanceChanges: vi.fn().mockResolvedValue({ value: mockVaultBalanceChanges })
	} as unknown as RaindexVault;

	render(VaultBalanceChangesTable, {
		props: {
			vault: mockVault
		},
		context: new Map([['$$_queryClient', queryClient]])
	});

	await waitFor(() => {
		const rows = screen.getAllByTestId('bodyRow');
		expect(rows).toHaveLength(1);
	});
});

test('it shows the correct data in the table with combined Info column', async () => {
	const queryClient = new QueryClient();

	const mockVaultBalanceChanges: RaindexVaultBalanceChange[] = [createMockVaultBalanceChange()];
	const mockVault: RaindexVault = {
		id: 'vault1',
		getBalanceChanges: vi.fn().mockResolvedValue({ value: mockVaultBalanceChanges })
	} as unknown as RaindexVault;

	render(VaultBalanceChangesTable, {
		props: {
			vault: mockVault
		},
		context: new Map([['$$_queryClient', queryClient]])
	});

	await waitFor(() => {
		// Info column now contains both type badge and date
		const infoCell = screen.getByTestId('vaultBalanceChangesTableInfo');
		expect(infoCell).toHaveTextContent(VAULT_BALANCE_CHANGE_LABELS.withdrawal);
		expect(infoCell).toHaveTextContent(formatTimestampSecondsAsLocal(BigInt('1625247600')));

		// Transaction column now contains both Sender and Tx
		const txCell = screen.getByTestId('vaultBalanceChangesTableTx');
		expect(txCell).toHaveTextContent('Sender:');
		expect(txCell).toHaveTextContent('Tx:');

		// Balance Change column shows token symbol and amount
		const balanceChangeCell = screen.getByTestId('vaultBalanceChangesTableBalanceChange');
		expect(balanceChangeCell).toHaveTextContent('TKN1');
		expect(balanceChangeCell).toHaveTextContent('0.1');

		// New Balance column shows token symbol and amount
		const balanceCell = screen.getByTestId('vaultBalanceChangesTableBalance');
		expect(balanceCell).toHaveTextContent('TKN1');
		expect(balanceCell).toHaveTextContent('0.4');
	});
});

describe('type badge labels', () => {
	const testCases: { type: RaindexVaultBalanceChangeType; expectedLabel: string }[] = [
		{ type: 'deposit', expectedLabel: 'Deposit' },
		{ type: 'withdrawal', expectedLabel: 'Withdrawal' },
		{ type: 'takeOrder', expectedLabel: 'Take order' },
		{ type: 'clear', expectedLabel: 'Clear' },
		{ type: 'clearBounty', expectedLabel: 'Clear Bounty' }
	];

	testCases.forEach(({ type, expectedLabel }) => {
		test(`displays ${type} with correct label`, async () => {
			const queryClient = new QueryClient();

			const mockVaultBalanceChanges: RaindexVaultBalanceChange[] = [
				createMockVaultBalanceChange({ type })
			];
			const mockVault: RaindexVault = {
				id: 'vault1',
				getBalanceChanges: vi.fn().mockResolvedValue({ value: mockVaultBalanceChanges })
			} as unknown as RaindexVault;

			render(VaultBalanceChangesTable, {
				props: {
					vault: mockVault
				},
				context: new Map([['$$_queryClient', queryClient]])
			});

			await waitFor(() => {
				const infoCell = screen.getByTestId('vaultBalanceChangesTableInfo');
				expect(infoCell).toHaveTextContent(expectedLabel);
			});
		});
	});
});

describe('type badge colors', () => {
	const colorTestCases: { type: RaindexVaultBalanceChangeType; expectedColor: string }[] = [
		{ type: 'deposit', expectedColor: 'green' },
		{ type: 'withdrawal', expectedColor: 'yellow' },
		{ type: 'takeOrder', expectedColor: 'blue' },
		{ type: 'clear', expectedColor: 'pink' },
		{ type: 'clearBounty', expectedColor: 'purple' }
	];

	colorTestCases.forEach(({ type, expectedColor }) => {
		test(`displays ${type} with ${expectedColor} badge`, async () => {
			const queryClient = new QueryClient();

			const mockVaultBalanceChanges: RaindexVaultBalanceChange[] = [
				createMockVaultBalanceChange({ type })
			];
			const mockVault: RaindexVault = {
				id: 'vault1',
				getBalanceChanges: vi.fn().mockResolvedValue({ value: mockVaultBalanceChanges })
			} as unknown as RaindexVault;

			render(VaultBalanceChangesTable, {
				props: {
					vault: mockVault
				},
				context: new Map([['$$_queryClient', queryClient]])
			});

			await waitFor(() => {
				const badge = document.querySelector('#type-tx1');
				expect(badge?.className).toContain(`bg-${expectedColor}-100`);
			});
		});
	});
});

test('renders the filter dropdown', async () => {
	const queryClient = new QueryClient();
	const mockVault: RaindexVault = {
		id: 'vault1',
		getBalanceChanges: vi.fn().mockResolvedValue({ value: [] })
	} as unknown as RaindexVault;

	render(VaultBalanceChangesTable, {
		props: { vault: mockVault },
		context: new Map([['$$_queryClient', queryClient]])
	});

	expect(screen.getByText('Vault balance changes')).toBeInTheDocument();
	expect(screen.getByText('Change Type')).toBeInTheDocument();
	expect(screen.getByTestId('dropdown-checkbox-button')).toBeInTheDocument();
});

test('calls getBalanceChanges with undefined initially', async () => {
	const queryClient = new QueryClient();
	const mockGetBalanceChanges = vi.fn().mockResolvedValue({ value: [] });
	const mockVault: RaindexVault = {
		id: 'vault1',
		getBalanceChanges: mockGetBalanceChanges
	} as unknown as RaindexVault;

	render(VaultBalanceChangesTable, {
		props: { vault: mockVault },
		context: new Map([['$$_queryClient', queryClient]])
	});

	await waitFor(() => {
		expect(mockGetBalanceChanges).toHaveBeenCalledWith(1, undefined);
	});
});
