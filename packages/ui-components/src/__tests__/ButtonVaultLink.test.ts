import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import ButtonVaultLink from '../lib/components/ButtonVaultLink.svelte';
import type { RaindexVault } from '@rainlanguage/raindex';

describe('ButtonVaultLink', () => {
	const mockVault = {
		id: '123',
		vaultId: BigInt(1000),
		balance: BigInt('1000000000000000000'),
		token: {
			name: 'Test Token',
			symbol: 'TEST',
			decimals: '18'
		}
	} as unknown as RaindexVault;

	it('should render vault information correctly', () => {
		render(ButtonVaultLink, {
			props: {
				tokenVault: mockVault,
				chainId: 1,
				raindexAddress: '0x00'
			}
		});

		const vaultLink = screen.getByTestId('vault-link');
		expect(vaultLink).toBeTruthy();
		expect(vaultLink).toHaveTextContent('Test Token');
		expect(vaultLink).toHaveTextContent('TEST');
	});

	it('should set the link id attribute correctly', () => {
		render(ButtonVaultLink, {
			props: {
				tokenVault: mockVault,
				chainId: 1,
				raindexAddress: '0x00'
			}
		});

		const linkElement = screen.getByRole('link');
		expect(linkElement).toHaveAttribute('id', `token-info-${mockVault.vaultId}`);
	});
});
