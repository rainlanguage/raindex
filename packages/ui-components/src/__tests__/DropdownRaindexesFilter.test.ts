import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { get, writable } from 'svelte/store';
import DropdownRaindexesFilter from '../lib/components/dropdown/DropdownRaindexesFilter.svelte';
import { expect, test, describe, beforeEach, vi, type Mock } from 'vitest';
import type { Address } from '@rainlanguage/raindex';
import { useRaindexClient } from '$lib/hooks/useRaindexClient';

vi.mock('$lib/hooks/useRaindexClient', () => ({
	useRaindexClient: vi.fn()
}));

const mockRaindexesData = new Map([
	[
		'raindex1',
		{
			key: 'raindex1',
			address: '0x1234567890123456789012345678901234567890',
			label: 'Raindex One',
			network: { chainId: 1 }
		}
	],
	[
		'raindex2',
		{
			key: 'raindex2',
			address: '0x2345678901234567890123456789012345678901',
			label: 'Raindex Two',
			network: { chainId: 1 }
		}
	],
	[
		'raindex3',
		{
			key: 'raindex3',
			address: '0x3456789012345678901234567890123456789012',
			label: null,
			network: { chainId: 137 }
		}
	]
]);

describe('DropdownRaindexesFilter', () => {
	let activeRaindexAddresses: ReturnType<typeof writable<Address[]>>;

	beforeEach(() => {
		activeRaindexAddresses = writable([]);

		(useRaindexClient as Mock).mockReturnValue({
			getAllRaindexes: vi.fn(() => ({
				value: mockRaindexesData,
				error: undefined
			}))
		});
	});

	describe('Empty state', () => {
		test('displays empty message when no raindexes available', async () => {
			(useRaindexClient as Mock).mockReturnValue({
				getAllRaindexes: vi.fn(() => ({
					value: new Map(),
					error: undefined
				}))
			});

			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-raindexes-filter-button'));

			await waitFor(() => {
				expect(screen.getByText('No raindexes available')).toBeInTheDocument();
			});
		});

		test('displays custom empty message', async () => {
			(useRaindexClient as Mock).mockReturnValue({
				getAllRaindexes: vi.fn(() => ({
					value: new Map(),
					error: undefined
				}))
			});

			const customEmptyMessage = 'Raindex list is empty';

			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: [],
					selectedChainIds: [],
					emptyMessage: customEmptyMessage
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-raindexes-filter-button'));

			await waitFor(() => {
				expect(screen.getByText(customEmptyMessage)).toBeInTheDocument();
			});
		});
	});

	describe('Error state', () => {
		test('displays error message when getAllRaindexes returns error', async () => {
			const errorMessage = 'Failed to load raindexes';
			(useRaindexClient as Mock).mockReturnValue({
				getAllRaindexes: vi.fn(() => ({
					value: undefined,
					error: { readableMsg: errorMessage }
				}))
			});

			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-raindexes-filter-button'));

			await waitFor(() => {
				expect(screen.getByText(`Cannot load raindexes list: ${errorMessage}`)).toBeInTheDocument();
			});
		});
	});

	describe('Selected raindexes display', () => {
		test('displays "Select raindexes" when no raindexes are selected', () => {
			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: [],
					selectedChainIds: []
				}
			});

			expect(screen.getByText('Select raindexes')).toBeInTheDocument();
		});

		test('displays "All raindexes" when all raindexes are selected', () => {
			const allAddresses = Array.from(mockRaindexesData.values()).map(
				(raindex) => raindex.address
			) as Address[];

			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: allAddresses,
					selectedChainIds: []
				}
			});

			expect(screen.getByText('All raindexes')).toBeInTheDocument();
		});

		test('displays custom all label when all raindexes are selected', () => {
			const allAddresses = Array.from(mockRaindexesData.values()).map(
				(raindex) => raindex.address
			) as Address[];
			const customAllLabel = 'Everything selected';

			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: allAddresses,
					selectedChainIds: [],
					allLabel: customAllLabel
				}
			});

			expect(screen.getByText(customAllLabel)).toBeInTheDocument();
		});

		test('displays count when one raindex is selected', () => {
			const selectedAddress = Array.from(mockRaindexesData.values())[0].address as Address;

			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: [selectedAddress],
					selectedChainIds: []
				}
			});

			expect(screen.getByText('1 raindex')).toBeInTheDocument();
		});

		test('displays plural count when multiple raindexes are selected', () => {
			const selectedAddresses = Array.from(mockRaindexesData.values())
				.slice(0, 2)
				.map((raindex) => raindex.address) as Address[];

			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: selectedAddresses,
					selectedChainIds: []
				}
			});

			expect(screen.getByText('2 raindexes')).toBeInTheDocument();
		});

		test('updates selected raindexes when checkbox is clicked', async () => {
			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-raindexes-filter-button'));

			const checkboxes = screen.getAllByTestId('dropdown-raindexes-filter-option');
			await fireEvent.click(checkboxes[0]);

			await waitFor(() => {
				const selected = get(activeRaindexAddresses);
				expect(selected.length).toBe(1);
			});
		});

		test('shows selected raindexes as checked', async () => {
			const selectedAddress = Array.from(mockRaindexesData.values())[0].address as Address;

			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: [selectedAddress],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-raindexes-filter-button'));

			await waitFor(() => {
				expect(screen.getByText('1 raindex')).toBeInTheDocument();
			});
		});
	});

	describe('Chain filtering', () => {
		test('shows all raindexes when selectedChainIds is empty', async () => {
			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-raindexes-filter-button'));

			await waitFor(() => {
				const checkboxes = screen.getAllByTestId('dropdown-raindexes-filter-option');
				expect(checkboxes.length).toBe(3);
			});
		});

		test('filters raindexes by selected chain ID', async () => {
			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: [],
					selectedChainIds: [1]
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-raindexes-filter-button'));

			await waitFor(() => {
				const checkboxes = screen.getAllByTestId('dropdown-raindexes-filter-option');
				expect(checkboxes.length).toBe(2);
			});
		});

		test('shows raindexes from multiple selected chains', async () => {
			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: [],
					selectedChainIds: [1, 137]
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-raindexes-filter-button'));

			await waitFor(() => {
				const checkboxes = screen.getAllByTestId('dropdown-raindexes-filter-option');
				expect(checkboxes.length).toBe(3);
			});
		});
	});

	describe('Display format', () => {
		test('displays label with truncated address when label exists', async () => {
			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-raindexes-filter-button'));

			await waitFor(() => {
				expect(screen.getByText(/Raindex One/)).toBeInTheDocument();
				expect(screen.getByText(/0x1234\.\.\.7890/)).toBeInTheDocument();
			});
		});

		test('displays only truncated address when no label', async () => {
			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: [],
					selectedChainIds: [137]
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-raindexes-filter-button'));

			await waitFor(() => {
				expect(screen.getByText(/0x3456\.\.\.9012/)).toBeInTheDocument();
			});
		});

		test('displays network name next to each raindex', async () => {
			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-raindexes-filter-button'));

			await waitFor(() => {
				expect(screen.getAllByText('Ethereum').length).toBeGreaterThan(0);
				expect(screen.getByText('Polygon')).toBeInTheDocument();
			});
		});
	});

	describe('Search and keyboard navigation', () => {
		test('filters raindexes based on search term (label)', async () => {
			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-raindexes-filter-button'));

			const searchInput = screen.getByTestId('raindexes-filter-search');
			await fireEvent.input(searchInput, { target: { value: 'One' } });

			await waitFor(() => {
				const options = screen.getAllByTestId('dropdown-raindexes-filter-option');
				expect(options).toHaveLength(1);
				expect(screen.getByText(/Raindex One/)).toBeInTheDocument();
			});
		});

		test('filters raindexes based on search term (address)', async () => {
			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-raindexes-filter-button'));

			const searchInput = screen.getByTestId('raindexes-filter-search');
			await fireEvent.input(searchInput, { target: { value: '0x1234' } });

			await waitFor(() => {
				const options = screen.getAllByTestId('dropdown-raindexes-filter-option');
				expect(options).toHaveLength(1);
			});
		});

		test('shows "No raindexes match your search" when search yields no results', async () => {
			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-raindexes-filter-button'));

			const searchInput = screen.getByTestId('raindexes-filter-search');
			await fireEvent.input(searchInput, { target: { value: 'NONEXISTENT' } });

			await waitFor(() => {
				expect(screen.getByText('No raindexes match your search')).toBeInTheDocument();
			});
		});

		test('keyboard navigation works correctly (ArrowDown/ArrowUp)', async () => {
			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-raindexes-filter-button'));

			const searchInput = screen.getByTestId('raindexes-filter-search');

			await fireEvent.keyDown(searchInput, { key: 'ArrowDown' });

			await waitFor(() => {
				const options = screen.getAllByTestId('dropdown-raindexes-filter-option');
				const secondItemLabel = options[1].closest('label');
				expect(secondItemLabel).toHaveClass('bg-blue-100', 'dark:bg-blue-900');
			});

			await fireEvent.keyDown(searchInput, { key: 'ArrowUp' });

			await waitFor(() => {
				const options = screen.getAllByTestId('dropdown-raindexes-filter-option');
				const firstItemLabel = options[0].closest('label');
				expect(firstItemLabel).toHaveClass('bg-blue-100', 'dark:bg-blue-900');
			});
		});

		test('Enter key selects highlighted raindex', async () => {
			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-raindexes-filter-button'));

			const searchInput = screen.getByTestId('raindexes-filter-search');

			await fireEvent.input(searchInput, { target: { value: 'One' } });

			await waitFor(() => {
				const options = screen.getAllByTestId('dropdown-raindexes-filter-option');
				expect(options).toHaveLength(1);
			});

			await fireEvent.keyDown(searchInput, { key: 'Enter' });

			await waitFor(() => {
				const selected = get(activeRaindexAddresses);
				expect(selected).toContain('0x1234567890123456789012345678901234567890'.toLowerCase());
			});
		});

		test('Escape key clears search', async () => {
			render(DropdownRaindexesFilter, {
				props: {
					activeRaindexAddresses,
					selectedRaindexAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-raindexes-filter-button'));

			const searchInput = screen.getByTestId('raindexes-filter-search') as HTMLInputElement;

			await fireEvent.input(searchInput, { target: { value: 'One' } });
			expect(searchInput.value).toBe('One');

			await fireEvent.keyDown(searchInput, { key: 'Escape' });

			await waitFor(() => {
				expect(searchInput.value).toBe('');
			});
		});
	});
});
