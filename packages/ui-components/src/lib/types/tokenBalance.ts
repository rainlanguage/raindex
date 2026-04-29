import type { AccountBalance } from '@rainlanguage/raindex';

export interface TokenBalance {
	value: AccountBalance;
	loading: boolean;
	error: string;
}
