import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { get, writable, readable } from 'svelte/store';
import DropdownTokensFilter from '../lib/components/dropdown/DropdownTokensFilter.svelte';
import { expect, test, describe, beforeEach } from 'vitest';
import type { Address, RaindexVaultToken } from '@rainlanguage/raindex';
import type { QueryObserverResult } from '@tanstack/svelte-query';

describe('DropdownTokensFilter', () => {
	let activeTokens: ReturnType<typeof writable<Address[]>>;
	let selectedTokens: Address[];

	const mockTokensData = [
		{
			id: 'TOKEN1',
			address: '0x1234567890123456789012345678901234567890',
			symbol: 'TOKEN1',
			name: 'Test Token 1',
			decimals: BigInt(18)
		},
		{
			id: 'TOKEN2',
			address: '0x2345678901234567890123456789012345678901',
			symbol: 'TOKEN2',
			name: 'Test Token 2',
			decimals: BigInt(18)
		},
		{
			id: 'ETH',
			address: '0x3456789012345678901234567890123456789012',
			symbol: 'ETH',
			name: 'Ethereum',
			decimals: BigInt(18)
		}
	] as unknown as RaindexVaultToken[];

	beforeEach(() => {
		activeTokens = writable([]);
		selectedTokens = [];
	});

	function createMockTokensQuery(
		data: RaindexVaultToken[] | undefined = undefined,
		isLoading = false,
		isError = false,
		error: Error | null = null
	) {
		return readable({
			data,
			isLoading,
			isError,
			error
		} as QueryObserverResult<RaindexVaultToken[], Error>);
	}

	describe('Loading state', () => {
		test('displays loading message when tokens are loading', async () => {
			const tokensQuery = createMockTokensQuery(undefined, true);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					activeTokens,
					selectedTokens
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			await waitFor(() => {
				expect(screen.getByText('Loading tokens...')).toBeInTheDocument();
			});
		});

		test('displays custom loading message', async () => {
			const tokensQuery = createMockTokensQuery(undefined, true);
			const customLoadingMessage = 'Fetching token data...';

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					activeTokens,
					selectedTokens,
					loadingMessage: customLoadingMessage
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			await waitFor(() => {
				expect(screen.getByText(customLoadingMessage)).toBeInTheDocument();
			});
		});
	});

	describe('Error state', () => {
		test('displays error message when query fails', async () => {
			const errorMessage = 'Network connection failed';
			const tokensQuery = createMockTokensQuery(undefined, false, true, new Error(errorMessage));

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					activeTokens,
					selectedTokens
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			await waitFor(() => {
				expect(screen.getByText(`Cannot load tokens list: ${errorMessage}`)).toBeInTheDocument();
			});
		});

		test('displays generic error message when error has no message', async () => {
			const tokensQuery = createMockTokensQuery(undefined, false, true, new Error());

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					activeTokens,
					selectedTokens
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			await waitFor(() => {
				expect(screen.getByText('Cannot load tokens list: Unknown error')).toBeInTheDocument();
			});
		});
	});

	describe('Empty state', () => {
		test('displays empty message when no tokens available', async () => {
			const tokensQuery = createMockTokensQuery([]);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					activeTokens,
					selectedTokens
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			await waitFor(() => {
				expect(screen.getByText('No tokens available')).toBeInTheDocument();
			});
		});

		test('displays custom empty message', async () => {
			const tokensQuery = createMockTokensQuery([]);
			const customEmptyMessage = 'Token list is empty';

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					activeTokens,
					selectedTokens,
					emptyMessage: customEmptyMessage
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			await waitFor(() => {
				expect(screen.getByText(customEmptyMessage)).toBeInTheDocument();
			});
		});
	});

	describe('Selected tokens display', () => {
		test('displays "Select tokens" when no tokens are selected', () => {
			const tokensQuery = createMockTokensQuery(mockTokensData);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					activeTokens,
					selectedTokens: []
				}
			});

			expect(screen.getByText('Select tokens')).toBeInTheDocument();
		});

		test('displays "All tokens" when all tokens are selected', () => {
			const tokensQuery = createMockTokensQuery(mockTokensData);
			const allTokenAddresses = mockTokensData.map((t) => t.address).filter(Boolean) as Address[];

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					activeTokens,
					selectedTokens: allTokenAddresses
				}
			});

			expect(screen.getByText('All tokens')).toBeInTheDocument();
		});

		test('displays custom all label when all tokens are selected', () => {
			const tokensQuery = createMockTokensQuery(mockTokensData);
			const allTokenAddresses = mockTokensData.map((t) => t.address).filter(Boolean) as Address[];
			const customAllLabel = 'Everything selected';

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					activeTokens,
					selectedTokens: allTokenAddresses,
					allLabel: customAllLabel
				}
			});

			expect(screen.getByText(customAllLabel)).toBeInTheDocument();
		});

		test('displays count when some tokens are selected', () => {
			const tokensQuery = createMockTokensQuery(mockTokensData);
			const selectedAddresses = [mockTokensData[0].address] as Address[];

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					activeTokens,
					selectedTokens: selectedAddresses
				}
			});

			expect(screen.getByText('1 token')).toBeInTheDocument();
		});

		test('displays plural count when multiple tokens are selected', () => {
			const tokensQuery = createMockTokensQuery(mockTokensData);
			const selectedAddresses = [mockTokensData[0].address, mockTokensData[1].address] as Address[];

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					activeTokens,
					selectedTokens: selectedAddresses
				}
			});

			expect(screen.getByText('2 tokens')).toBeInTheDocument();
		});

		test('updates selected tokens when checkbox is clicked', async () => {
			const tokensQuery = createMockTokensQuery(mockTokensData);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					activeTokens,
					selectedTokens: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			const checkboxes = screen.getAllByTestId('dropdown-tokens-filter-option');
			await fireEvent.click(checkboxes[0]);

			await waitFor(() => {
				expect(get(activeTokens)).toContain(mockTokensData[0].address);
			});
		});

		test('shows selected tokens as checked', async () => {
			const tokensQuery = createMockTokensQuery(mockTokensData);
			const selectedAddress = mockTokensData[0].address;

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					activeTokens,
					selectedTokens: [selectedAddress]
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			await waitFor(() => {
				const checkboxes = screen.getAllByTestId('dropdown-tokens-filter-option');
				expect(checkboxes.length).toBeGreaterThan(0);

				// Check the text displayed shows that there's a selection
				expect(screen.getByText('1 token')).toBeInTheDocument();
			});
		});
	});

	describe('Search and selectedIndex behavior', () => {
		test('filters tokens based on search term', async () => {
			const tokensQuery = createMockTokensQuery(mockTokensData);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					activeTokens,
					selectedTokens: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			const searchInput = screen.getByTestId('tokens-filter-search');
			await fireEvent.input(searchInput, { target: { value: 'ETH' } });

			await waitFor(() => {
				const options = screen.getAllByTestId('dropdown-tokens-filter-option');
				expect(options).toHaveLength(1);
				expect(screen.getByText('ETH')).toBeInTheDocument();
			});
		});

		test('shows "No tokens match your search" when search yields no results', async () => {
			const tokensQuery = createMockTokensQuery(mockTokensData);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					activeTokens,
					selectedTokens: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			const searchInput = screen.getByTestId('tokens-filter-search');
			await fireEvent.input(searchInput, { target: { value: 'NONEXISTENT' } });

			await waitFor(() => {
				expect(screen.getByText('No tokens match your search')).toBeInTheDocument();
			});
		});

		test('selectedIndex transitions correctly from empty to non-empty results', async () => {
			const tokensQuery = createMockTokensQuery(mockTokensData);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					activeTokens,
					selectedTokens: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			const searchInput = screen.getByTestId('tokens-filter-search');

			// First search for something that doesn't exist
			await fireEvent.input(searchInput, { target: { value: 'NONEXISTENT' } });

			await waitFor(() => {
				expect(screen.getByText('No tokens match your search')).toBeInTheDocument();
			});

			// Then search for something that exists - first item should be highlighted
			await fireEvent.input(searchInput, { target: { value: 'ETH' } });

			await waitFor(() => {
				const options = screen.getAllByTestId('dropdown-tokens-filter-option');
				expect(options).toHaveLength(1);
				// Check if the parent label has the selected styling
				const parentLabel = options[0].closest('label');
				expect(parentLabel).toHaveClass('bg-blue-100', 'dark:bg-blue-900');
			});
		});

		test('selectedIndex transitions correctly from non-empty to empty results', async () => {
			const tokensQuery = createMockTokensQuery(mockTokensData);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					activeTokens,
					selectedTokens: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			const searchInput = screen.getByTestId('tokens-filter-search');

			// First search for something that exists
			await fireEvent.input(searchInput, { target: { value: 'TOKEN' } });

			await waitFor(() => {
				const options = screen.getAllByTestId('dropdown-tokens-filter-option');
				expect(options.length).toBeGreaterThan(0);
			});

			// Then search for something that doesn't exist
			await fireEvent.input(searchInput, { target: { value: 'NONEXISTENT' } });

			await waitFor(() => {
				expect(screen.getByText('No tokens match your search')).toBeInTheDocument();
				// No options should be present
				expect(screen.queryAllByTestId('dropdown-tokens-filter-option')).toHaveLength(0);
			});
		});

		test('keyboard navigation works correctly', async () => {
			const tokensQuery = createMockTokensQuery(mockTokensData);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					activeTokens,
					selectedTokens: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			const searchInput = screen.getByTestId('tokens-filter-search');

			// Arrow down should move selection
			await fireEvent.keyDown(searchInput, { key: 'ArrowDown' });

			await waitFor(() => {
				const options = screen.getAllByTestId('dropdown-tokens-filter-option');
				// Check if second item's parent label is highlighted (index 1)
				const secondItemLabel = options[1].closest('label');
				expect(secondItemLabel).toHaveClass('bg-blue-100', 'dark:bg-blue-900');
			});

			// Arrow up should move selection back
			await fireEvent.keyDown(searchInput, { key: 'ArrowUp' });

			await waitFor(() => {
				const options = screen.getAllByTestId('dropdown-tokens-filter-option');
				// Check if first item's parent label is highlighted again (index 0)
				const firstItemLabel = options[0].closest('label');
				expect(firstItemLabel).toHaveClass('bg-blue-100', 'dark:bg-blue-900');
			});
		});

		test('Enter key selects highlighted token', async () => {
			const tokensQuery = createMockTokensQuery(mockTokensData);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					activeTokens,
					selectedTokens: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			const searchInput = screen.getByTestId('tokens-filter-search');

			// Search for a specific token
			await fireEvent.input(searchInput, { target: { value: 'ETH' } });

			await waitFor(() => {
				const options = screen.getAllByTestId('dropdown-tokens-filter-option');
				expect(options).toHaveLength(1);
			});

			// Press Enter to select the highlighted token
			await fireEvent.keyDown(searchInput, { key: 'Enter' });

			await waitFor(() => {
				const ethToken = mockTokensData.find((t) => t.symbol === 'ETH');
				expect(get(activeTokens)).toContain(ethToken?.address);
			});
		});

		test('Escape key clears search', async () => {
			const tokensQuery = createMockTokensQuery(mockTokensData);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					activeTokens,
					selectedTokens: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			const searchInput = screen.getByTestId('tokens-filter-search') as HTMLInputElement;

			// Enter search term
			await fireEvent.input(searchInput, { target: { value: 'ETH' } });
			expect(searchInput.value).toBe('ETH');

			// Press Escape to clear
			await fireEvent.keyDown(searchInput, { key: 'Escape' });

			await waitFor(() => {
				expect(searchInput.value).toBe('');
			});
		});
	});
});
