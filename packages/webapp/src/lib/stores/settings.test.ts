import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { selectedChainIds, validChainIds } from './settings';

describe('settings store', () => {
	beforeEach(() => {
		// Reset the shared module-singleton stores and persisted state between tests.
		localStorage.clear();
		selectedChainIds.set([]);
		validChainIds.set([]);
	});

	describe('selectedChainIds pruning via validChainIds', () => {
		it('prunes chain IDs that are absent from validChainIds when validChainIds is populated', () => {
			// Chains 1, 137, 42161 are selected.
			selectedChainIds.set([1, 137, 42161]);

			// The valid configuration contains only chains 1 and 42161.
			validChainIds.set([1, 42161]);

			// 137 is stale (not in the current configuration) and must be removed.
			expect(get(selectedChainIds)).toEqual([1, 42161]);
		});

		it('retains every selected chain ID that is present in validChainIds', () => {
			selectedChainIds.set([1, 137]);

			validChainIds.set([1, 137, 42161]);

			// Nothing is stale, so the selection is left untouched.
			expect(get(selectedChainIds)).toEqual([1, 137]);
		});

		it('removes all selected chain IDs when none of them are valid', () => {
			selectedChainIds.set([999, 1234]);

			validChainIds.set([1, 137]);

			expect(get(selectedChainIds)).toEqual([]);
		});

		it('leaves selectedChainIds untouched while validChainIds is empty', () => {
			// While validChainIds is empty, the persisted selection is retained in full.
			selectedChainIds.set([1, 137, 42161]);

			validChainIds.set([]);

			expect(get(selectedChainIds)).toEqual([1, 137, 42161]);
		});

		it('re-prunes when validChainIds changes again to a narrower set', () => {
			selectedChainIds.set([1, 137, 42161]);

			validChainIds.set([1, 137, 42161]);
			expect(get(selectedChainIds)).toEqual([1, 137, 42161]);

			// Narrowing the valid set prunes the stale 137.
			validChainIds.set([1, 42161]);
			expect(get(selectedChainIds)).toEqual([1, 42161]);
		});

		it('seeds selectedChainIds with all validChainIds when selectedChainIds is empty', () => {
			// Fresh user: no prior chain selection.
			// When the client discovers which chains are available, select them all so
			// orders are visible without requiring a wallet connection first.
			validChainIds.set([1, 137, 42161]);

			expect(get(selectedChainIds)).toEqual([1, 137, 42161]);
		});

		it('does not re-seed selectedChainIds when validChainIds updates and selectedChainIds is already set', () => {
			// Returning user: their explicit selection must not be clobbered when
			// validChainIds is refreshed (e.g. after a re-init).
			selectedChainIds.set([1, 42161]);

			validChainIds.set([1, 137, 42161]);

			// 137 must NOT be auto-added; user chose 1 and 42161.
			expect(get(selectedChainIds)).toEqual([1, 42161]);
		});
	});

	describe('selectedChainIds localStorage persistence', () => {
		it('serializes selected chain IDs to localStorage as JSON', () => {
			selectedChainIds.set([1, 137, 42161]);

			expect(localStorage.getItem('settings.selectedChainIds')).toBe(
				JSON.stringify([1, 137, 42161])
			);
		});

		it('round-trips the selected chain IDs through serialize/deserialize', () => {
			selectedChainIds.set([10, 8453]);

			const persisted = localStorage.getItem('settings.selectedChainIds');
			expect(persisted).not.toBeNull();
			expect(JSON.parse(persisted as string)).toEqual([10, 8453]);
		});

		it('persists the pruned (not the stale) selection back to localStorage', () => {
			selectedChainIds.set([1, 137, 42161]);

			validChainIds.set([1, 42161]);

			// The stale 137 must not survive in persisted storage either.
			expect(JSON.parse(localStorage.getItem('settings.selectedChainIds') as string)).toEqual([
				1, 42161
			]);
		});
	});
});
