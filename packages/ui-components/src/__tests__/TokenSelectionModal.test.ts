import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import TokenSelectionModal from '../lib/components/deployment/TokenSelectionModal.svelte';
import type { ComponentProps } from 'svelte';
import type { ExtendedTokenInfo, RaindexOrderBuilder } from '@rainlanguage/orderbook';
import { useRaindexOrderBuilder } from '$lib/hooks/useRaindexOrderBuilder';

type TokenSelectionModalProps = ComponentProps<TokenSelectionModal>;

const mockTokens: ExtendedTokenInfo[] = [
	{
		key: 'token1',
		address: '0x1234567890123456789012345678901234567890',
		name: 'Test Token 1',
		symbol: 'TEST1',
		decimals: 18,
		chainId: 1,
		logoUri: 'https://example.com/token1-logo.png'
	},
	{
		key: 'token2',
		address: '0x0987654321098765432109876543210987654321',
		name: 'Another Token',
		symbol: 'ANOTHER',
		decimals: 6,
		chainId: 1,
		logoUri: undefined
	}
];

const mockBuilder: RaindexOrderBuilder = {
	getAllTokens: vi.fn().mockResolvedValue({
		value: mockTokens
	})
} as unknown as RaindexOrderBuilder;

vi.mock('../lib/hooks/useRaindexOrderBuilder', () => ({
	useRaindexOrderBuilder: vi.fn()
}));

describe('TokenSelectionModal', () => {
	let mockOnSelect: ReturnType<typeof vi.fn>;

	const defaultProps: TokenSelectionModalProps = {
		selectedToken: null,
		onSelect: vi.fn()
	};

	beforeEach(() => {
		mockOnSelect = vi.fn();
		(useRaindexOrderBuilder as Mock).mockReturnValue(mockBuilder);
		vi.clearAllMocks();
	});

	it('renders modal button with default text when no token is selected', () => {
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		expect(screen.getByText('Select a token...')).toBeInTheDocument();
	});

	it('renders modal button with selected token info when token is selected', () => {
		const selectedToken = mockTokens[0];
		render(TokenSelectionModal, {
			...defaultProps,
			selectedToken,
			onSelect: mockOnSelect
		});

		expect(screen.getByText('Test Token 1 (TEST1)')).toBeInTheDocument();
	});

	it('opens modal when button is clicked', async () => {
		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		expect(screen.getByText('Select a token')).toBeInTheDocument();
	});

	it('shows search input in modal', async () => {
		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		expect(screen.getByPlaceholderText('Search tokens...')).toBeInTheDocument();
	});

	it('loads tokens on mount', async () => {
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await userEvent.click(button);

		await waitFor(() => {
			expect(mockBuilder.getAllTokens).toHaveBeenCalledWith(undefined);
		});
	});

	it('shows tokens in modal after loading', async () => {
		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		await waitFor(() => {
			expect(screen.getByText('Test Token 1')).toBeInTheDocument();
			expect(screen.getByText('Another Token')).toBeInTheDocument();
		});
	});

	it('calls onSelect when token is clicked', async () => {
		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		await waitFor(() => {
			expect(screen.getByText('Test Token 1')).toBeInTheDocument();
		});

		const tokenItem = screen.getByText('Test Token 1');
		await user.click(tokenItem);

		expect(mockOnSelect).toHaveBeenCalledWith(mockTokens[0]);
	});

	it('calls API with search term when searching', async () => {
		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		const searchInput = screen.getByPlaceholderText('Search tokens...');
		await user.type(searchInput, 'TEST');

		await waitFor(() => {
			expect(mockBuilder.getAllTokens).toHaveBeenCalledWith('TEST');
		});
	});

	it('shows loading state while searching', async () => {
		const mockBuilderWithDelay = {
			getAllTokens: vi
				.fn()
				.mockImplementation(
					() => new Promise((resolve) => setTimeout(() => resolve({ value: mockTokens }), 100))
				)
		} as unknown as RaindexOrderBuilder;

		(useRaindexOrderBuilder as Mock).mockReturnValue(mockBuilderWithDelay);

		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		const searchInput = screen.getByPlaceholderText('Search tokens...');
		await user.type(searchInput, 'TEST');

		expect(screen.getByText('Searching tokens...')).toBeInTheDocument();
	});

	it('shows no results message when search returns empty', async () => {
		const mockBuilderNoResults = {
			getAllTokens: vi.fn().mockResolvedValue({ value: [] })
		} as unknown as RaindexOrderBuilder;

		(useRaindexOrderBuilder as Mock).mockReturnValue(mockBuilderNoResults);

		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		await waitFor(() => {
			expect(screen.getByText('No tokens found matching your search.')).toBeInTheDocument();
		});
	});

	it('clears search when clear button is clicked', async () => {
		const mockBuilderNoResults = {
			getAllTokens: vi
				.fn()
				.mockResolvedValueOnce({ value: [] })
				.mockResolvedValueOnce({ value: mockTokens })
		} as unknown as RaindexOrderBuilder;

		(useRaindexOrderBuilder as Mock).mockReturnValue(mockBuilderNoResults);

		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		await waitFor(() => {
			expect(screen.getByText('No tokens found matching your search.')).toBeInTheDocument();
		});

		const clearButton = screen.getByText('Clear search');
		await user.click(clearButton);

		await waitFor(() => {
			expect(mockBuilderNoResults.getAllTokens).toHaveBeenCalledWith(undefined);
		});
	});

	it('automatically focuses search input when modal opens', async () => {
		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		await waitFor(() => {
			const searchInput = screen.getByPlaceholderText('Search tokens...');
			expect(searchInput).toHaveFocus();
		});
	});

	it('displays token logo image when logoUri is provided', async () => {
		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		await waitFor(() => {
			const logoImage = screen.getByAltText('TEST1 logo');
			expect(logoImage).toBeInTheDocument();
			expect(logoImage).toHaveAttribute('src', 'https://example.com/token1-logo.png');
		});
	});

	it('does not display logo image when logoUri is not provided', async () => {
		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		await waitFor(() => {
			expect(screen.getByText('Another Token')).toBeInTheDocument();
		});

		expect(screen.queryByAltText('ANOTHER logo')).not.toBeInTheDocument();
	});

	it('displays symbol fallback for tokens without logo', async () => {
		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		await waitFor(() => {
			expect(screen.getByText('AN')).toBeInTheDocument();
		});
	});

	it('includes logoUri in token data when selecting a token with logo', async () => {
		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		await waitFor(() => {
			expect(screen.getByText('Test Token 1')).toBeInTheDocument();
		});

		const tokenItem = screen.getByText('Test Token 1');
		await user.click(tokenItem);

		expect(mockOnSelect).toHaveBeenCalledWith(
			expect.objectContaining({
				logoUri: 'https://example.com/token1-logo.png'
			})
		);
	});

	it('falls back to symbol initials when image fails to load', async () => {
		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		await waitFor(() => {
			expect(screen.getByAltText('TEST1 logo')).toBeInTheDocument();
		});

		const logoImage = screen.getByAltText('TEST1 logo');
		logoImage.dispatchEvent(new Event('error'));

		await waitFor(() => {
			expect(screen.queryByAltText('TEST1 logo')).not.toBeInTheDocument();
			expect(screen.getByText('TE')).toBeInTheDocument();
		});
	});
});
