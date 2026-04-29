// TODO: Issue #1989
// import { render } from '@testing-library/svelte';
import { test } from 'vitest';
// import { expect, test, vi } from 'vitest';
// import { QueryClient } from '@tanstack/svelte-query';
// import VaultBalanceChart from '../lib/components/charts/VaultBalanceChart.svelte';
// import type { RaindexVault } from '@rainlanguage/raindex';
// import { writable } from 'svelte/store';
// import type { ComponentProps } from 'svelte';

// type VaultBalanceChartProps = ComponentProps<VaultBalanceChart>;

// vi.mock('../lib/components/charts/TanstackLightweightChartLine.svelte', async () => {
// 	const MockLightweightChart = (await import('../lib/__mocks__/MockComponent.svelte')).default;
// 	return { default: MockLightweightChart };
// });

test('calls getVaultBalanceChanges with correct arguments', async () => {
	// 	const queryClient = new QueryClient();
	// 	const mockVault: RaindexVault = {
	// 		id: 'vault1',
	// 		getBalanceChanges: vi.fn()
	// 	} as unknown as RaindexVault;
	// 	render(VaultBalanceChart, {
	// 		props: {
	// 			vault: mockVault,
	// 			lightweightChartsTheme: writable({})
	// 		} as VaultBalanceChartProps,
	// 		context: new Map([['$$_queryClient', queryClient]])
	// 	});
	// 	expect(mockVault.getBalanceChanges).toHaveBeenCalledWith(1);
});
