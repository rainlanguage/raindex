import type {
	RaindexVaultBalanceChangeType,
	VaultBalanceChangeFilter
} from '@rainlanguage/raindex';

export const VAULT_BALANCE_CHANGE_LABELS: Record<RaindexVaultBalanceChangeType, string> = {
	deposit: 'Deposit',
	withdrawal: 'Withdrawal',
	takeOrder: 'Take order',
	clear: 'Clear',
	clearBounty: 'Clear Bounty',
	unknown: 'Unknown'
};

export const VAULT_BALANCE_CHANGE_FILTER_LABELS: Record<VaultBalanceChangeFilter, string> = {
	deposit: 'Deposit',
	withdrawal: 'Withdrawal',
	takeOrder: 'Take order',
	clear: 'Clear',
	clearBounty: 'Clear Bounty'
};

export function labelForVaultBalanceChangeType(type: RaindexVaultBalanceChangeType): string {
	return VAULT_BALANCE_CHANGE_LABELS[type] ?? type;
}
