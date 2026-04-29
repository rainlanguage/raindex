import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/test/matchers';
import type { RaindexOrder, RaindexVaultVolume } from '@rainlanguage/raindex';
import OrderVaultsVolTable from '../lib/components/tables/OrderVaultsVolTable.svelte';
import { QueryClient } from '@tanstack/svelte-query';

const mockVaultsVol = [
	{
		id: BigInt('1'),
		token: {
			id: 'output_token',
			address: '0xoutput_token',
			name: 'output_token',
			symbol: 'output_token',
			decimals: BigInt('0')
		},
		details: {
			formattedTotalIn: '1',
			formattedTotalOut: '2',
			formattedTotalVol: '3',
			formattedNetVol: '-1'
		}
	},
	{
		id: BigInt('2'),
		token: {
			id: 'output_token',
			address: '0xoutput_token',
			name: 'output_token',
			symbol: 'output_token',
			decimals: BigInt('0')
		},
		details: {
			formattedTotalIn: '2',
			formattedTotalOut: '5',
			formattedTotalVol: '7',
			formattedNetVol: '-3'
		}
	}
] as unknown as RaindexVaultVolume[];

test('renders table with correct data', async () => {
	const queryClient = new QueryClient();
	const mockOrder: RaindexOrder = {
		id: '1',
		getVaultsVolume: vi.fn().mockResolvedValue({ value: mockVaultsVol })
	} as unknown as RaindexOrder;
	render(OrderVaultsVolTable, {
		context: new Map([['$$_queryClient', queryClient]]),
		props: { order: mockOrder }
	});
	await waitFor(async () => {
		const rows = screen.getAllByTestId('total-in');
		for (let i = 0; i < mockVaultsVol.length; i++) {
			const display = mockVaultsVol[i].details.formattedTotalIn;
			expect(rows[i]).toHaveTextContent(display);
		}
	});
	await waitFor(async () => {
		const rows = screen.getAllByTestId('total-out');
		for (let i = 0; i < mockVaultsVol.length; i++) {
			const display = mockVaultsVol[i].details.formattedTotalOut;
			expect(rows[i]).toHaveTextContent(display);
		}
	});
	await waitFor(async () => {
		const rows = screen.getAllByTestId('net-vol');
		for (let i = 0; i < mockVaultsVol.length; i++) {
			const display = mockVaultsVol[i].details.formattedNetVol;
			expect(rows[i]).toHaveTextContent(display);
		}
	});
	await waitFor(async () => {
		const rows = screen.getAllByTestId('total-vol');
		for (let i = 0; i < mockVaultsVol.length; i++) {
			const display = mockVaultsVol[i].details.formattedTotalVol;
			expect(rows[i]).toHaveTextContent(display);
		}
	});
});
