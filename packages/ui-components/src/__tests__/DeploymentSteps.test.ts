import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import DeploymentSteps from '../lib/components/deployment/DeploymentSteps.svelte';
import { DotrainOrderGui, type ScenarioCfg } from '@rainlanguage/orderbook';
import type { ComponentProps } from 'svelte';
import { readable, writable } from 'svelte/store';
import type { AppKit } from '@reown/appkit';
import type { GuiDeploymentCfg, RaindexClient } from '@rainlanguage/orderbook';
import userEvent from '@testing-library/user-event';
import { useGui } from '$lib/hooks/useGui';
import { useAccount } from '$lib/providers/wallet/useAccount';
import type { Account } from '$lib/types/account';
import { useRaindexClient } from '$lib/hooks/useRaindexClient';

vi.mock('@rainlanguage/orderbook', () => ({
	DotrainOrderGui: vi.fn()
}));

const { mockConnectedStore } = await vi.hoisted(() => import('../lib/__mocks__/stores'));

vi.mock('$lib/hooks/useGui', () => ({
	useGui: vi.fn()
}));

vi.mock('$lib/hooks/useRaindexClient', () => ({
	useRaindexClient: vi.fn()
}));

vi.mock('$lib/providers/wallet/useAccount', () => ({
	useAccount: vi.fn()
}));

type DeploymentStepsProps = ComponentProps<DeploymentSteps>;

const mockDeployment = {
	key: 'flare-sflr-wflr',
	name: 'SFLR<>WFLR on Flare',
	description: 'Rotate sFLR (Sceptre staked FLR) and WFLR on Flare.',
	deposits: [],
	fields: [],
	select_tokens: [],
	deployment: {
		key: 'flare-sflr-wflr',
		scenario: {
			key: 'flare',
			bindings: {}
		} as ScenarioCfg,
		order: {
			key: 'flare-sflr-wflr',
			network: {
				key: 'flare',
				'chain-id': 14,
				'network-id': 14,
				rpc: 'https://rpc.ankr.com/flare',
				label: 'Flare',
				currency: 'FLR'
			},
			rainlang: {
				key: 'flare',
				bindings: {}
			} as ScenarioCfg,
			order: {
				key: 'flare-sflr-wflr',
				network: {
					key: 'flare',
					'chain-id': 14,
					'network-id': 14,
					rpc: 'https://rpc.ankr.com/flare',
					label: 'Flare',
					currency: 'FLR'
				},
				address: '0x0'
			},
			orderbook: {
				id: 'flare',
				address: '0x0'
			},
			inputs: [],
			outputs: []
		}
	}
} as unknown as GuiDeploymentCfg;

const mockOnDeploy = vi.fn();

const defaultProps: DeploymentStepsProps = {
	orderDetail: {
		name: 'SFLR<>WFLR on Flare',
		description: 'Rotate sFLR (Sceptre staked FLR) and WFLR on Flare.',
		short_description: 'Rotate sFLR (Sceptre staked FLR) and WFLR on Flare.'
	},
	deployment: mockDeployment,
	wagmiConnected: mockConnectedStore,
	appKitModal: writable({} as AppKit),
	onDeploy: mockOnDeploy,
	account: readable('0x123')
} as DeploymentStepsProps;

describe('DeploymentSteps', () => {
	let guiInstance: DotrainOrderGui;
	let mockGui: DotrainOrderGui;

	beforeEach(() => {
		vi.clearAllMocks();

		// Create a mock instance with all the methods
		guiInstance = {
			areAllTokensSelected: vi.fn().mockReturnValue({ value: false }),
			getSelectTokens: vi.fn().mockReturnValue({ value: [] }),
			getCurrentDeployment: vi.fn().mockReturnValue(mockDeployment),
			getAllFieldDefinitions: vi.fn().mockReturnValue({ value: [] }),
			hasAnyDeposit: vi.fn().mockReturnValue({ value: false }),
			hasAnyVaultId: vi.fn().mockReturnValue(false),
			getAllTokenInfos: vi.fn().mockResolvedValue({ value: [] }),
			getAllTokens: vi.fn().mockResolvedValue({
				value: [
					{
						key: 'token1',
						address: '0x1234567890123456789012345678901234567890',
						name: 'Test Token 1',
						symbol: 'TEST1',
						decimals: 18
					},
					{
						key: 'token2',
						address: '0x0987654321098765432109876543210987654321',
						name: 'Another Token',
						symbol: 'ANOTHER',
						decimals: 6
					}
				]
			}),
			getCurrentDeploymentDetails: vi.fn().mockReturnValue({
				value: {
					name: 'Test Deployment',
					description: 'This is a test deployment description'
				}
			}),
			getTokenInfo: vi.fn().mockResolvedValue({
				value: {
					name: 'Test Token',
					symbol: 'TEST',
					address: '0x123',
					decimals: 18
				}
			}),
			isSelectTokenSet: vi.fn().mockReturnValue({ value: false }),
			setSelectToken: vi.fn(),
			unsetSelectToken: vi.fn(),
			getDeploymentTransactionArgs: vi.fn(),
			getAccountBalance: vi.fn().mockResolvedValue({
				value: '1000000000000000000'
			})
		} as unknown as DotrainOrderGui;

		mockGui = guiInstance;
		vi.mocked(useGui).mockReturnValue(mockGui);

		// Set default mock return value for useAccount
		vi.mocked(useAccount).mockReturnValue({
			account: writable(null),
			matchesAccount: vi.fn()
		});
	});

	it('shows deployment details when provided', async () => {
		render(DeploymentSteps, { props: defaultProps });

		await waitFor(() => {
			expect(screen.getByText('SFLR<>WFLR on Flare')).toBeInTheDocument();
		});
	});

	it('shows select tokens section when tokens need to be selected', async () => {
		(mockGui.getSelectTokens as Mock).mockReturnValue({
			value: ['token1', 'token2']
		});

		render(DeploymentSteps, {
			props: defaultProps
		});

		await waitFor(() => {
			expect(screen.getByText('Select Tokens')).toBeInTheDocument();
			expect(
				screen.getByText('Select the tokens that you want to use in your order.')
			).toBeInTheDocument();
		});
	});

	it('shows wallet connect button when all required fields are filled, but no account exists', async () => {
		const mockSelectTokens = [
			{ key: 'token1', name: 'Token 1', description: undefined },
			{ key: 'token2', name: 'Token 2', description: undefined }
		];

		// Set up specific mocks for this test
		(mockGui.getSelectTokens as Mock).mockReturnValue({
			value: mockSelectTokens
		});
		(mockGui.getTokenInfo as Mock).mockImplementation(() => {});
		(mockGui.areAllTokensSelected as Mock).mockReturnValue({ value: true });
		(mockGui.isSelectTokenSet as Mock).mockReturnValue({ value: false });
		(mockGui.setSelectToken as Mock).mockImplementation(() => {});
		(mockGui.getCurrentDeployment as Mock).mockReturnValue({
			value: {
				deployment: {
					order: {
						inputs: [],
						outputs: []
					}
				},
				deposits: []
			}
		});

		(mockGui.getAllTokenInfos as Mock).mockResolvedValue({
			value: [
				{
					address: '0x1',
					decimals: 18,
					name: 'Token 1',
					symbol: 'TKN1'
				},
				{
					address: '0x2',
					decimals: 18,
					name: 'Token 2',
					symbol: 'TKN2'
				}
			]
		});

		render(DeploymentSteps, { props: { ...defaultProps, account: readable(null) } });

		await waitFor(() => {
			expect(screen.getByText('Connect')).toBeInTheDocument();
		});
	});

	it('shows deploy button when all required fields are filled, and account is connected', async () => {
		const mockSelectTokens = [
			{ key: 'token1', name: 'Token 1', description: undefined },
			{ key: 'token2', name: 'Token 2', description: undefined }
		];

		// Set up specific mocks for this test
		(mockGui.getSelectTokens as Mock).mockReturnValue({
			value: mockSelectTokens
		});
		(mockGui.getTokenInfo as Mock).mockImplementation(() => {});
		(mockGui.areAllTokensSelected as Mock).mockReturnValue({ value: true });
		(mockGui.isSelectTokenSet as Mock).mockReturnValue({ value: false });
		(mockGui.setSelectToken as Mock).mockImplementation(() => {});
		(mockGui.getCurrentDeployment as Mock).mockReturnValue({
			value: {
				deployment: {
					order: {
						inputs: [],
						outputs: []
					}
				},
				deposits: []
			}
		});

		(mockGui.getAllTokenInfos as Mock).mockResolvedValue({
			value: [
				{
					address: '0x1',
					decimals: 18,
					name: 'Token 1',
					symbol: 'TKN1'
				},
				{
					address: '0x2',
					decimals: 18,
					name: 'Token 2',
					symbol: 'TKN2'
				}
			]
		});

		render(DeploymentSteps, { props: defaultProps });

		await waitFor(() => {
			expect(screen.getByText('Deploy Order')).toBeInTheDocument();
		});
	});
	it('refreshes field descriptions when tokens change', async () => {
		const mockSelectTokens = [
			{ key: 'token1', name: 'Token 1', description: undefined },
			{ key: 'token2', name: 'Token 2', description: undefined }
		];

		// Set up specific mocks for this test
		(mockGui.getSelectTokens as Mock).mockReturnValue({
			value: mockSelectTokens
		});
		(mockGui.getTokenInfo as Mock).mockImplementation(() => {});
		(mockGui.areAllTokensSelected as Mock).mockReturnValue({ value: true });
		(mockGui.isSelectTokenSet as Mock).mockReturnValue({ value: false });
		(mockGui.setSelectToken as Mock).mockImplementation(() => {});
		(mockGui.getCurrentDeployment as Mock).mockReturnValue({
			value: {
				deployment: {
					order: {
						inputs: [],
						outputs: []
					}
				},
				deposits: []
			}
		});

		(mockGui.getAllTokenInfos as Mock).mockResolvedValue({
			value: [
				{
					address: '0x1',
					decimals: 18,
					name: 'Token 1',
					symbol: 'TKN1'
				},
				{
					address: '0x2',
					decimals: 18,
					name: 'Token 2',
					symbol: 'TKN2'
				}
			]
		});

		const user = userEvent.setup();

		render(DeploymentSteps, {
			props: defaultProps
		});

		await waitFor(() => {
			expect(mockGui.areAllTokensSelected).toHaveBeenCalled();
		});

		await waitFor(() => {
			expect(screen.getByText('Select Tokens')).toBeInTheDocument();
			expect(screen.getByText('Token 1')).toBeInTheDocument();
			expect(screen.getByText('Token 2')).toBeInTheDocument();
		});

		const selectTokenInput = screen.getByText('Token 1');
		(mockGui.getTokenInfo as Mock).mockResolvedValue({
			value: {
				address: '0x1',
				decimals: 18,
				name: 'Token 1',
				symbol: 'TKN1'
			}
		});
		await user.type(selectTokenInput, '0x1');

		const selectTokenOutput = screen.getByText('Token 2');
		(mockGui.getTokenInfo as Mock).mockResolvedValue({
			value: {
				address: '0x2',
				decimals: 18,
				name: 'Token 2',
				symbol: 'TKN2'
			}
		});
		await user.type(selectTokenOutput, '0x2');

		await waitFor(() => {
			expect(mockGui.getAllTokenInfos).toHaveBeenCalled();
		});

		const customAddressButtons = screen.getAllByTestId('custom-mode-button');
		await user.click(customAddressButtons[customAddressButtons.length - 1]);
		const customInputs = screen.getAllByPlaceholderText('Enter token address (0x...)');
		const lastCustomInput = customInputs[customInputs.length - 1];
		(mockGui.getTokenInfo as Mock).mockResolvedValue({
			value: {
				address: '0x3',
				decimals: 18,
				name: 'Token 3',
				symbol: 'TKN3'
			}
		});
		await user.type(lastCustomInput, '0x3');

		(mockGui.getAllTokenInfos as Mock).mockResolvedValue({
			value: [
				{
					address: '0x3',
					decimals: 18,
					name: 'Token 3',
					symbol: 'TKN3'
				},
				{
					address: '0x2',
					decimals: 18,
					name: 'Token 2',
					symbol: 'TKN2'
				}
			]
		});

		await waitFor(() => {
			expect(mockGui.getAllTokenInfos).toHaveBeenCalled();
		});
	});

	it('passes correct arguments to onDeploy prop', async () => {
		// Override the mock for this test
		guiInstance.areAllTokensSelected = vi.fn().mockReturnValue({ value: true });
		vi.mocked(useGui).mockReturnValue(guiInstance);

		const mockRaindexClient = {} as unknown as RaindexClient;
		vi.mocked(useRaindexClient).mockReturnValue(mockRaindexClient);

		const propsWithMockHandlers = {
			...defaultProps,
			account: readable('0xTestAccount') as Account
		};

		const user = userEvent.setup();
		render(DeploymentSteps, { props: propsWithMockHandlers });

		const deployButton = screen.getByText('Deploy Order');
		await user.click(deployButton);

		await waitFor(() => {
			expect(mockOnDeploy).toHaveBeenCalledTimes(1);

			const [raindexClient, guiArg] = mockOnDeploy.mock.calls[0];

			expect(guiArg).toBe(mockGui);
			expect(raindexClient).toBe(mockRaindexClient);
		});
	});

	it('calls getTokenInfoAndFetchBalance for each selected token on account change', async () => {
		const mockSelectTokens = [
			{ key: 'token1', name: 'Token 1', description: undefined },
			{ key: 'token2', name: 'Token 2', description: undefined }
		];

		(mockGui.getSelectTokens as Mock).mockReturnValue({
			value: mockSelectTokens
		});

		const accountStore = writable<`0x${string}` | null>('0x123');

		const propsWithAccountStore = {
			...defaultProps,
			account: accountStore
		};

		render(DeploymentSteps, { props: propsWithAccountStore });

		await waitFor(() => {
			expect(mockGui.getSelectTokens).toHaveBeenCalled();
		});
		vi.clearAllMocks();

		accountStore.set('0x456');

		await waitFor(() => {
			expect(mockGui.getTokenInfo).toHaveBeenCalledTimes(2);
			expect(mockGui.getTokenInfo).toHaveBeenCalledWith('token1');
			expect(mockGui.getTokenInfo).toHaveBeenCalledWith('token2');
		});
		await waitFor(() => {
			expect(mockGui.getAccountBalance).toHaveBeenCalledTimes(2);
		});
	});

	it('clears token balances when account becomes null', async () => {
		const mockSelectTokens = [{ key: 'token1', name: 'Token 1', description: undefined }];

		(mockGui.getSelectTokens as Mock).mockReturnValue({
			value: mockSelectTokens
		});

		const accountStore = writable<`0x${string}` | null>('0x123');

		const propsWithAccountStore = {
			...defaultProps,
			account: accountStore
		};

		render(DeploymentSteps, { props: propsWithAccountStore });

		await waitFor(() => {
			expect(mockGui.getSelectTokens).toHaveBeenCalled();
		});

		vi.clearAllMocks();

		accountStore.set(null);

		// await new Promise((resolve) => setTimeout(resolve, 100));
		expect(mockGui.getTokenInfo).not.toHaveBeenCalled();
		expect(mockGui.getAccountBalance).not.toHaveBeenCalled();
	});
});
