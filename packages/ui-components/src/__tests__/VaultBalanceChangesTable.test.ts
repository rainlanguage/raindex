import { render, screen } from '@testing-library/svelte';
import { test } from 'vitest';
import { expect } from '../lib/test/matchers';
import VaultBalanceChangesTable from '../lib/components/tables/VaultBalanceChangesTable.svelte';
import type { RaindexVaultBalanceChange } from '@rainlanguage/orderbook';
import { formatTimestampSecondsAsLocal } from '../lib/services/time';

test('renders the vault list table with correct data', async () => {
	const mockVaultBalanceChanges: RaindexVaultBalanceChange[] = [
		{
			__typename: 'Withdrawal',
			amount: BigInt(1000),
			oldBalance: BigInt(5000),
			newBalance: BigInt(4000),
			timestamp: BigInt(1625247600),
			vaultId: BigInt(100),
			token: {
				id: 'token1',
				address: '0xTokenAddress1',
				name: 'Token1',
				symbol: 'TKN1',
				decimals: '18'
			},
			transaction: {
				id: 'tx1',
				from: '0xUser1',
				timestamp: BigInt(1625247600),
				blockNumber: BigInt(1234567890)
			},
			orderbook: '0x00'
		}
	] as unknown as RaindexVaultBalanceChange[];

	render(VaultBalanceChangesTable, {
		props: {
			data: mockVaultBalanceChanges
		}
	});

	const rows = screen.getAllByTestId('bodyRow');
	expect(rows).toHaveLength(1);
});

test('it shows the correct data in the table', async () => {
	const mockVaultBalanceChanges: RaindexVaultBalanceChange[] = [
		{
			type: 'withdrawal',
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
			orderbook: '0x00'
		}
	] as unknown as RaindexVaultBalanceChange[];

	render(VaultBalanceChangesTable, {
		props: {
			data: mockVaultBalanceChanges
		}
	});

	expect(screen.getByTestId('vaultBalanceChangesTableDate')).toHaveTextContent(
		formatTimestampSecondsAsLocal(BigInt('1625247600'))
	);
	expect(screen.getByTestId('vaultBalanceChangesTableFrom')).toHaveTextContent('0xUse...User1');
	expect(screen.getByTestId('vaultBalanceChangesTableTx')).toHaveTextContent('tx1');
	expect(screen.getByTestId('vaultBalanceChangesTableBalanceChange')).toHaveTextContent('0.1 TKN1');
	expect(screen.getByTestId('vaultBalanceChangesTableBalance')).toHaveTextContent('0.4 TKN1');
	expect(screen.getByTestId('vaultBalanceChangesTableType')).toHaveTextContent('withdrawal');
});
