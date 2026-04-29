import { render, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import TokenIOInput from '../lib/components/deployment/TokenIOInput.svelte';
import type { ComponentProps } from 'svelte';
import { AccountBalance, RaindexOrderBuilder, Float } from '@rainlanguage/orderbook';
import { useRaindexOrderBuilder } from '$lib/hooks/useRaindexOrderBuilder';
import type { TokenBalance } from '$lib/types/tokenBalance';

vi.mock('@rainlanguage/orderbook', async (importOriginal) => {
	return {
		...(await importOriginal()),
		RaindexOrderBuilder: vi.fn()
	};
});

vi.mock('$lib/hooks/useRaindexOrderBuilder', () => ({
	useRaindexOrderBuilder: vi.fn()
}));

type TokenIOInputComponentProps = ComponentProps<TokenIOInput>;

describe('TokenInput', () => {
	let builderInstance: RaindexOrderBuilder;
	let mockStateUpdateCallback: Mock;
	let mockProps: TokenIOInputComponentProps;
	let outputMockProps: TokenIOInputComponentProps;

	const mockInput = {
		token: {
			address: '0x123',
			key: 'test'
		}
	};
	const mockTokenInfo = {
		value: {
			symbol: 'MOCK',
			name: 'Mock Token',
			decimals: 18
		}
	};

	beforeEach(() => {
		vi.clearAllMocks();

		// Create a mock instance with all the methods
		builderInstance = {
			getTokenInfo: vi.fn().mockResolvedValue(mockTokenInfo),
			setVaultId: vi.fn().mockImplementation(() => {
				mockStateUpdateCallback();
			}),
			getCurrentDeployment: vi.fn().mockResolvedValue({
				deployment: {
					order: {
						inputs: [mockInput]
					}
				}
			}),
			getVaultIds: vi.fn().mockReturnValue({
				value: new Map([
					['input', new Map([['test', 'vault1']])],
					['output', new Map([['test', 'vault2']])]
				])
			})
		} as unknown as RaindexOrderBuilder;

		mockStateUpdateCallback = vi.fn();

		(useRaindexOrderBuilder as Mock).mockReturnValue(builderInstance);

		mockProps = {
			label: 'Input',
			vault: mockInput,
			tokenBalances: new Map()
		} as unknown as TokenIOInputComponentProps;
		outputMockProps = {
			label: 'Output',
			vault: mockInput,
			tokenBalances: new Map()
		} as unknown as TokenIOInputComponentProps;
	});

	it('renders with correct label and no token symbol', () => {
		const { getByText } = render(TokenIOInput, mockProps);
		expect(getByText('Input')).toBeInTheDocument();
	});

	it('renders input field with correct placeholder', () => {
		const { getByPlaceholderText } = render(TokenIOInput, mockProps);
		const input = getByPlaceholderText('Enter vault ID');
		expect(input).toBeInTheDocument();
	});

	it('displays the correct vault ID value', async () => {
		const { getByText } = render(TokenIOInput, mockProps);
		await waitFor(() => {
			expect(getByText('MOCK vault ID')).toBeInTheDocument();
		});
	});

	it('calls setVaultId when input changes', async () => {
		const input = render(TokenIOInput, mockProps).getByPlaceholderText('Enter vault ID');
		await fireEvent.input(input, { target: { value: 'vault1' } });
		expect(builderInstance.setVaultId).toHaveBeenCalledWith('input', 'test', 'vault1');
		expect(mockStateUpdateCallback).toHaveBeenCalledTimes(1);
	});

	it('calls setVaultId on output vault when input changes', async () => {
		const input = render(TokenIOInput, outputMockProps).getByPlaceholderText('Enter vault ID');
		await fireEvent.input(input, { target: { value: 'vault2' } });
		expect(builderInstance.setVaultId).toHaveBeenCalledWith('output', 'test', 'vault2');
		expect(mockStateUpdateCallback).toHaveBeenCalledTimes(1);
	});

	it('handles missing token info gracefully', () => {
		const propsWithUnknownToken = {
			...mockProps,
			vault: { token: { address: '0x789' } }
		};
		const { getByText } = render(
			TokenIOInput,
			propsWithUnknownToken as unknown as TokenIOInputComponentProps
		);
		expect(getByText('Input')).toBeInTheDocument();
	});

	describe('Balance Display', () => {
		it('passes token balance to VaultIdInformation component', async () => {
			const tokenBalances = new Map<string, TokenBalance>();
			tokenBalances.set('test', {
				value: {
					balance: Float.parse('1').value,
					formattedBalance: '1'
				} as AccountBalance,
				loading: false,
				error: ''
			});

			const propsWithBalance = {
				...mockProps,
				tokenBalances
			};

			const { findByText } = render(TokenIOInput, propsWithBalance);

			const labelWithSymbol = await findByText('Input (MOCK)');
			expect(labelWithSymbol).toBeInTheDocument();
		});

		it('passes loading balance state to VaultIdInformation component', async () => {
			const tokenBalances = new Map<string, TokenBalance>();
			tokenBalances.set('test', {
				value: {
					balance: Float.parse('0').value,
					formattedBalance: '0'
				} as AccountBalance,
				loading: true,
				error: ''
			});

			const propsWithLoadingBalance = {
				...mockProps,
				tokenBalances
			};

			const { findByText } = render(TokenIOInput, propsWithLoadingBalance);

			const labelWithSymbol = await findByText('Input (MOCK)');
			expect(labelWithSymbol).toBeInTheDocument();
		});

		it('passes balance error state to VaultIdInformation component', async () => {
			const tokenBalances = new Map<string, TokenBalance>();
			tokenBalances.set('test', {
				value: {
					balance: Float.parse('0').value,
					formattedBalance: '0'
				} as AccountBalance,
				loading: false,
				error: 'Network error'
			});

			const propsWithErrorBalance = {
				...mockProps,
				tokenBalances
			};

			const { findByText } = render(TokenIOInput, propsWithErrorBalance);

			const labelWithSymbol = await findByText('Input (MOCK)');
			expect(labelWithSymbol).toBeInTheDocument();
		});

		it('handles missing token balance gracefully', async () => {
			const propsWithoutBalance = {
				...mockProps,
				tokenBalances: new Map() // Empty map
			};

			const { findByText } = render(TokenIOInput, propsWithoutBalance);

			const labelWithSymbol = await findByText('Input (MOCK)');
			expect(labelWithSymbol).toBeInTheDocument();
		});
	});

	it('fetches and displays token symbol when token key is present', async () => {
		const propsWithTokenKey = {
			...mockProps,
			vault: {
				token: {
					key: '0x456'
				}
			}
		} as unknown as TokenIOInputComponentProps;

		const { findByText } = render(TokenIOInput, propsWithTokenKey);

		const labelWithSymbol = await findByText('Input (MOCK)');
		expect(labelWithSymbol).toBeInTheDocument();
		expect(builderInstance.getTokenInfo).toHaveBeenCalledWith('0x456');
	});
});
