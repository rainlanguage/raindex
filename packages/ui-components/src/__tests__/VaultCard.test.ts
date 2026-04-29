import { render, screen } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import VaultCard from '../lib/components/VaultCard.svelte';
import type { RaindexVault } from '@rainlanguage/raindex';
import userEvent from '@testing-library/user-event';

// Mock the navigation
vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

const mockVault: RaindexVault = {
	id: '0x1234567890abcdef1234567890abcdef12345678',
	chainId: 1,
	raindex: '0x2222222222222222222222222222222222222222',
	token: {
		symbol: 'ETH',
		name: 'Ethereum',
		address: '0x0000000000000000000000000000000000000000',
		decimals: 18
	},
	formattedBalance: '1.5'
} as unknown as RaindexVault;

describe('VaultCard', () => {
	it('renders vault information correctly', () => {
		render(VaultCard, {
			vault: mockVault
		});

		expect(screen.getByTestId('vault-card')).toBeInTheDocument();
		expect(screen.getByText('ETH')).toBeInTheDocument();
		expect(screen.getByText('1.5')).toBeInTheDocument();
	});

	it('navigates to vault details when clicked', async () => {
		const { goto } = await import('$app/navigation');

		render(VaultCard, {
			vault: mockVault
		});

		const vaultCard = screen.getByTestId('vault-card');
		await userEvent.click(vaultCard);

		expect(goto).toHaveBeenCalledWith(
			'/vaults/1-0x2222222222222222222222222222222222222222-0x1234567890abcdef1234567890abcdef12345678'
		);
	});

	it('displays different token symbols correctly', () => {
		const daiVault = {
			...mockVault,
			chainId: 137,
			token: {
				...mockVault.token,
				symbol: 'DAI'
			},
			formattedBalance: '2500.0'
		} as unknown as RaindexVault;

		render(VaultCard, {
			vault: daiVault
		});

		expect(screen.getByText('DAI')).toBeInTheDocument();
		expect(screen.getByText('2500.0')).toBeInTheDocument();
	});

	it('navigates with correct chain ID', async () => {
		const { goto } = await import('$app/navigation');

		const polygonVault = {
			...mockVault,
			chainId: 137
		} as unknown as RaindexVault;

		render(VaultCard, {
			vault: polygonVault
		});

		const vaultCard = screen.getByTestId('vault-card');
		await userEvent.click(vaultCard);

		expect(goto).toHaveBeenCalledWith(
			'/vaults/137-0x2222222222222222222222222222222222222222-0x1234567890abcdef1234567890abcdef12345678'
		);
	});

	it('has proper accessibility attributes', () => {
		render(VaultCard, {
			vault: mockVault
		});

		const button = screen.getByTestId('vault-card');
		expect(button).toHaveAttribute('type', 'button');
		expect(button.tagName).toBe('BUTTON');
	});

	it('displays formatted balance correctly', () => {
		const vaultWithLongBalance = {
			...mockVault,
			formattedBalance: '1,234,567.89'
		} as unknown as RaindexVault;

		render(VaultCard, {
			vault: vaultWithLongBalance
		});

		expect(screen.getByText('1,234,567.89')).toBeInTheDocument();
	});
});
