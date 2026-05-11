import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import DepositInput from '../lib/components/deployment/DepositInput.svelte';
import type { OrderBuilderDepositCfg } from '@rainlanguage/orderbook';
import type { ComponentProps } from 'svelte';
import { RaindexOrderBuilder } from '@rainlanguage/orderbook';
import { useRaindexOrderBuilder } from '$lib/hooks/useRaindexOrderBuilder';
type DepositInputProps = ComponentProps<DepositInput>;

vi.mock('@rainlanguage/orderbook', () => ({
	RaindexOrderBuilder: vi.fn()
}));

vi.mock('$lib/hooks/useRaindexOrderBuilder', () => ({
	useRaindexOrderBuilder: vi.fn()
}));

describe('DepositInput', () => {
	let mockStateUpdateCallback: Mock;
	let builderInstance: RaindexOrderBuilder;

	const mockDeposit: OrderBuilderDepositCfg = {
		token: { address: '0x123', key: 'TEST', symbol: 'TEST' },
		presets: ['100', '200', '300']
	} as unknown as OrderBuilderDepositCfg;

	beforeEach(() => {
		vi.clearAllMocks();

		builderInstance = {
			getDeposits: vi.fn().mockReturnValue({
				value: [{ token: 'output', amount: '10', address: '0x1234' }]
			}),
			setDeposit: vi.fn().mockImplementation(() => {
				mockStateUpdateCallback();
			}),
			getTokenInfo: vi.fn()
		} as unknown as RaindexOrderBuilder;

		mockStateUpdateCallback = vi.fn();
		(useRaindexOrderBuilder as Mock).mockReturnValue(builderInstance);
	});

	it('renders token name and presets', async () => {
		(builderInstance.getTokenInfo as Mock).mockResolvedValueOnce({
			value: {
				name: 'Test Token',
				symbol: 'TEST'
			}
		});

		const { getByText } = render(DepositInput, {
			props: {
				deposit: mockDeposit
			} as unknown as DepositInputProps
		});
		await waitFor(() => {
			expect(getByText(`Deposit amount (${mockDeposit.token?.symbol})`)).toBeTruthy();
			expect(getByText('100')).toBeTruthy();
			expect(getByText('200')).toBeTruthy();
			expect(getByText('300')).toBeTruthy();
		});
	});

	it('handles preset button clicks', async () => {
		const { getByText } = render(DepositInput, {
			props: {
				deposit: mockDeposit
			} as unknown as DepositInputProps
		});

		await fireEvent.click(getByText('100'));
		expect(builderInstance.setDeposit).toHaveBeenCalledWith('TEST', '100');
	});

	it('handles custom input changes and triggers state update', async () => {
		const { getByPlaceholderText } = render(DepositInput, {
			props: {
				deposit: mockDeposit,
				onStateUpdate: mockStateUpdateCallback
			} as unknown as DepositInputProps
		});

		const input = getByPlaceholderText('Enter deposit amount');
		await fireEvent.input(input, { target: { value: '150' } });

		expect(builderInstance.setDeposit).toHaveBeenCalledWith('TEST', '150');
		expect(mockStateUpdateCallback).toHaveBeenCalled();
	});
});
