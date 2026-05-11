import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { getContext } from 'svelte';
import {
	useRaindexOrderBuilder,
	RAINDEX_ORDER_BUILDER_CONTEXT_KEY
} from './useRaindexOrderBuilder';
import { DeploymentStepsError, DeploymentStepsErrorCode } from '../errors/DeploymentStepsError';

vi.mock('svelte', () => ({
	getContext: vi.fn()
}));

vi.spyOn(DeploymentStepsError, 'catch');

describe('useRaindexOrderBuilder hook', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	afterEach(() => {
		vi.clearAllMocks();
	});

	it('should return builder context when available', () => {
		const mockBuilder = {
			someMethod: vi.fn(),
			someProperty: 'value'
		};

		vi.mocked(getContext).mockReturnValue(mockBuilder);

		const result = useRaindexOrderBuilder();

		expect(getContext).toHaveBeenCalledWith(RAINDEX_ORDER_BUILDER_CONTEXT_KEY);
		expect(result).toBe(mockBuilder);
	});

	it('should call DeploymentStepsError.catch when builder context is not available', () => {
		vi.mocked(getContext).mockReturnValue(null);

		useRaindexOrderBuilder();

		expect(DeploymentStepsError.catch).toHaveBeenCalledWith(
			null,
			DeploymentStepsErrorCode.NO_BUILDER_PROVIDER
		);
	});
});
