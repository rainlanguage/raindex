import type { AccountCfg } from '@rainlanguage/raindex';

export function getAccountsAsOptions(accounts?: Map<string, AccountCfg>) {
	if (!accounts) return {};
	return Object.fromEntries(
		Array.from(accounts.entries()).map(([key, value]) => [key, value.address])
	);
}

if (import.meta.vitest) {
	const { expect, it, describe } = import.meta.vitest;

	describe('getAccountsAsOptions', () => {
		it('should return the correct accounts as options', () => {
			const accounts = new Map([
				['account1', { address: '0x1234567890abcdef', key: 'account1' }],
				['account2', { address: '0xabcdef1234567890', key: 'account2' }]
			]);
			const result = getAccountsAsOptions(accounts);
			expect(result).toEqual({
				account1: '0x1234567890abcdef',
				account2: '0xabcdef1234567890'
			});
		});
	});
}
