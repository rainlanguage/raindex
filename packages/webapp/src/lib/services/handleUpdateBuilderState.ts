import { pushState } from '$app/navigation';
import { debounce } from 'lodash';

export const pushBuilderStateToUrlHistory = debounce((serializedState: string) => {
	pushState(`?state=${serializedState}`, { serializedState });
}, 1000);

if (import.meta.vitest) {
	const { describe, it, expect, vi } = import.meta.vitest;

	// Mock pushState
	vi.mock('$app/navigation', () => ({
		pushState: vi.fn()
	}));

	describe('handleUpdateBuilderState', () => {
		beforeEach(() => {
			vi.clearAllMocks();
			vi.useFakeTimers();
		});

		afterEach(() => {
			vi.useRealTimers();
		});

		it('should push state to URL history when serializedState exists', async () => {
			const mockSerializedState = 'mockSerializedState123';
			pushBuilderStateToUrlHistory(mockSerializedState);

			// Fast-forward timers to trigger debounced function
			await vi.advanceTimersByTimeAsync(1000);

			expect(pushState).toHaveBeenCalledWith(`?state=${mockSerializedState}`, {
				serializedState: mockSerializedState
			});
		});

		it('should debounce multiple calls', async () => {
			const mockSerializedState = 'mockSerializedState123';

			// Call multiple times in quick succession
			pushBuilderStateToUrlHistory(mockSerializedState);
			pushBuilderStateToUrlHistory(mockSerializedState);
			pushBuilderStateToUrlHistory(mockSerializedState);

			await vi.advanceTimersByTimeAsync(1000);

			// Should only be called once due to debouncing
			expect(pushState).toHaveBeenCalledTimes(1);
			expect(pushState).toHaveBeenCalledWith(`?state=${mockSerializedState}`, {
				serializedState: mockSerializedState
			});
		});
	});
}
