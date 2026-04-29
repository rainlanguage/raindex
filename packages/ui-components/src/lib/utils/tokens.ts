import type { RaindexVaultToken } from '@rainlanguage/raindex';

export function getTokenDisplayName(token: RaindexVaultToken): string {
	return token.symbol || token.name || 'Unknown Token';
}
