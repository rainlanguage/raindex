import { vi, describe, it, expect, beforeAll, beforeEach, afterEach } from 'vitest';
import Page from './+page.svelte';
import { fireEvent, render, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import {
	useAccount,
	useToasts,
	useTransactions,
	type TransactionConfirmationProps
} from '@rainlanguage/ui-components';
import { readable, writable } from 'svelte/store';
import {
	DotrainRegistry,
	type RaindexOrderBuilder,
	type NameAndDescriptionCfg
} from '@rainlanguage/raindex';
import { REGISTRY_URL } from '$lib/constants';
import { retry, DEFAULT_MAX_RETRIES } from '$lib/retry';
import { handleTransactionConfirmationModal } from '$lib/services/modal';

const ACCOUNT = '0x999999cf1046e68e36E1aA2E0E07105eDDD1f08E';
const TOKEN1_ADDRESS = '0x000000000000012def132e61759048be5b5c6033';
const TOKEN2_ADDRESS = '0x00000000000007c8612ba63df8ddefd9e6077c97';

async function createRegistry(): Promise<DotrainRegistry> {
	return retry(async () => {
		const result = await DotrainRegistry.new(REGISTRY_URL);
		if (result.error) {
			throw new Error('Failed to create registry: ' + result.error.msg);
		}
		return result.value;
	});
}

async function getBuilder(
	rl: DotrainRegistry,
	serializedState?: string,
	stateCallback?: (state: string) => void
): Promise<RaindexOrderBuilder> {
	return retry(async () => {
		const result = await rl.getOrderBuilder(
			'fixed-limit',
			'base',
			serializedState,
			stateCallback ?? null
		);
		if (result.error) {
			throw new Error(result.error.readableMsg ?? result.error.msg);
		}
		return result.value;
	});
}

async function createConfiguredBuilder(
	rl: DotrainRegistry,
	stateCallback?: (state: string) => void
): Promise<RaindexOrderBuilder> {
	const builder = await getBuilder(rl, undefined, stateCallback);
	const token1Result = await builder.setSelectToken('token1', TOKEN1_ADDRESS);
	if (token1Result.error) {
		throw new Error('setSelectToken token1: ' + token1Result.error.msg);
	}
	const token2Result = await builder.setSelectToken('token2', TOKEN2_ADDRESS);
	if (token2Result.error) {
		throw new Error('setSelectToken token2: ' + token2Result.error.msg);
	}
	builder.setVaultId('output', 'token2', '234');
	builder.setVaultId('input', 'token1', '123');
	builder.setFieldValue('fixed-io', '10');
	return builder;
}

const { mockPageStore } = await vi.hoisted(() => import('@rainlanguage/ui-components'));

const { mockConnectedStore, mockAppKitModalStore, mockWagmiConfigStore } = await vi.hoisted(
	() => import('$lib/__mocks__/stores')
);

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		useTransactions: vi.fn(),
		useAccount: vi.fn(),
		useToasts: vi.fn()
	};
});

vi.mock('$lib/services/modal', async () => ({
	handleDisclaimerModal: vi.fn(async (props) => {
		const { DisclaimerModal } = await import('@rainlanguage/ui-components');
		new DisclaimerModal({
			target: document.body,
			props: { ...props, open: true }
		});
	}),
	handleTransactionConfirmationModal: vi.fn()
}));

vi.mock('$app/stores', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		page: mockPageStore
	};
});

vi.mock('$lib/stores/wagmi', () => ({
	connected: mockConnectedStore,
	appKitModal: mockAppKitModalStore,
	wagmiConfig: mockWagmiConfigStore
}));

// Shared across all describe blocks to avoid duplicate HTTP fetches to the
// remote registry/settings/token-list endpoints, which are rate-limited on CI.
let registry: DotrainRegistry;
beforeAll(async () => {
	registry = await createRegistry();
});

describe('Builder deployment args isolation tests', () => {
	it(
		'standalone builder without callback produces deployment args',
		async () => {
			const builder = await createConfiguredBuilder(registry);
			const result = await builder.getDeploymentTransactionArgs(ACCOUNT);
			expect(result.error).toBeUndefined();
			const args = result.value!;
			expect(args).toBeDefined();
			expect(args.deploymentCalldata).toBeDefined();
			expect(args.raindexAddress).toBeDefined();
			expect(args.chainId).toBeDefined();
		},
		{ timeout: 30000, retry: DEFAULT_MAX_RETRIES }
	);

	it(
		'standalone builder with noop state callback produces deployment args',
		async () => {
			const callback = vi.fn();
			const builder = await createConfiguredBuilder(registry, callback);
			const result = await builder.getDeploymentTransactionArgs(ACCOUNT);
			expect(result.error).toBeUndefined();
			const args = result.value!;
			expect(args).toBeDefined();
			expect(args.deploymentCalldata).toBeDefined();
			expect(callback).toHaveBeenCalled();
		},
		{ timeout: 30000, retry: DEFAULT_MAX_RETRIES }
	);

	it(
		'generateAddOrderCalldata works standalone',
		async () => {
			const builder = await createConfiguredBuilder(registry);
			const result = await builder.generateAddOrderCalldata();
			expect(result.error).toBeUndefined();
			expect(result.value).toBeDefined();
		},
		{ timeout: 30000, retry: DEFAULT_MAX_RETRIES }
	);

	it(
		'generateApprovalCalldatas works standalone',
		async () => {
			const builder = await createConfiguredBuilder(registry);
			const result = await builder.generateApprovalCalldatas(ACCOUNT);
			expect(result.error).toBeUndefined();
			expect(result.value).toBeDefined();
		},
		{ timeout: 30000, retry: DEFAULT_MAX_RETRIES }
	);

	it(
		'generateDepositCalldatas works standalone',
		async () => {
			const builder = await createConfiguredBuilder(registry);
			const result = await builder.generateDepositCalldatas();
			expect(result.error).toBeUndefined();
			expect(result.value).toBeDefined();
		},
		{ timeout: 30000, retry: DEFAULT_MAX_RETRIES }
	);

	it(
		'serializeState and restore produce deployment args',
		async () => {
			const builder1 = await createConfiguredBuilder(registry);
			const serialized = builder1.serializeState();
			expect(serialized.error).toBeUndefined();

			const builder2 = await getBuilder(registry, serialized.value);

			const result = await builder2.getDeploymentTransactionArgs(ACCOUNT);
			expect(result.error).toBeUndefined();
			const args = result.value!;
			expect(args).toBeDefined();
			expect(args.deploymentCalldata).toBeDefined();
		},
		{ timeout: 30000, retry: DEFAULT_MAX_RETRIES }
	);
});

describe('Full Deployment Tests', () => {
	interface TokenSelectionQueries {
		getAllByTestId: (id: string) => HTMLElement[];
		getAllByPlaceholderText: (text: string) => HTMLElement[];
		getByTestId: (id: string) => HTMLElement;
	}

	function findLockRegion(a: string, b: string): { prefixEnd: number; suffixEnd: number } {
		expect(a.length).toEqual(b.length);
		const length = a.length;
		// Find prefix end
		let prefixEnd = 0;
		while (prefixEnd < length && a[prefixEnd] === b[prefixEnd]) {
			prefixEnd++;
		}
		// Find suffix start
		let suffixEnd = length;
		while (suffixEnd > prefixEnd && a[suffixEnd - 1] === b[suffixEnd - 1]) {
			suffixEnd--;
		}
		return { prefixEnd, suffixEnd };
	}

	async function selectTokenByAddress(
		screen: TokenSelectionQueries,
		tokenIndex: number,
		address: string,
		successTestId: string
	) {
		await userEvent.click(screen.getAllByTestId('custom-mode-button')[tokenIndex]);
		const addressInputs = screen.getAllByPlaceholderText(
			'Enter token address (0x...)'
		) as HTMLInputElement[];
		await fireEvent.input(addressInputs[tokenIndex], {
			target: { value: address }
		});
		await waitFor(
			() => {
				expect(screen.getByTestId(successTestId)).toBeInTheDocument();
			},
			{ timeout: 30000 }
		);
	}

	beforeEach(async () => {
		vi.clearAllMocks();
		vi.mocked(useAccount).mockReturnValue({
			account: readable('0x999999cf1046e68e36E1aA2E0E07105eDDD1f08E'),
			matchesAccount: vi.fn()
		});
		vi.mocked(useToasts).mockReturnValue({
			removeToast: vi.fn(),
			toasts: writable([]),
			addToast: vi.fn(),
			errToast: vi.fn()
		});
		vi.mocked(useTransactions).mockReturnValue({
			// @ts-expect-error simple object
			manager: writable({}),
			transactions: readable()
		});
		mockConnectedStore.mockSetSubscribeValue(true);
		vi.mocked(handleTransactionConfirmationModal).mockResolvedValue({
			success: true,
			hash: '0xtesthash'
		});
	});

	afterEach(async () => {
		await new Promise((resolve) => setTimeout(resolve, 10000));
	});

	it(
		'Fixed limit order',
		async () => {
			const fixedLimitDeploymentDetails = registry.getDeploymentDetails('fixed-limit');
			if (fixedLimitDeploymentDetails.error) {
				throw new Error('Failed to get deployment details');
			}
			const deployment = fixedLimitDeploymentDetails.value.get('base') as NameAndDescriptionCfg;
			const fixedLimitOrderDetail = registry
				.getAllOrderDetails()
				.value?.valid.get('fixed-limit') as NameAndDescriptionCfg;

			mockPageStore.mockSetSubscribeValue({
				data: {
					orderName: 'fixed-limit',
					deployment: { key: 'base', ...deployment },
					registry,
					orderDetail: fixedLimitOrderDetail
				}
			});
			const screen = render(Page);

			// Wait for the builder provider to be in the document
			await waitFor(
				() => {
					expect(screen.getByTestId('builder-provider')).toBeInTheDocument();
				},
				{ timeout: 30000 }
			);

			await waitFor(
				() => {
					expect(screen.getAllByRole('button', { name: /chevron down solid/i }).length).toBe(2);
				},
				{ timeout: 30000 }
			);
			await selectTokenByAddress(screen, 0, TOKEN1_ADDRESS, 'select-token-success-token1');
			await new Promise((resolve) => setTimeout(resolve, 2000));

			await selectTokenByAddress(screen, 1, TOKEN2_ADDRESS, 'select-token-success-token2');
			// Wait for field definitions to render after token selection
			let customValueInput!: HTMLElement;
			await waitFor(
				() => {
					customValueInput = screen.getAllByPlaceholderText('Enter custom value')[0];
					expect(customValueInput).toBeInTheDocument();
				},
				{ timeout: 30000 }
			);
			// Allow async WASM operations (getTokenInfo, getAccountBalance) to
			// settle so their &self borrows are released before setFieldValue
			// takes &mut self.
			await new Promise((resolve) => setTimeout(resolve, 2000));
			await userEvent.clear(customValueInput);
			await userEvent.type(customValueInput, '10');

			const showAdvancedOptionsButton = screen.getByText('Show advanced options');
			await userEvent.click(showAdvancedOptionsButton);

			const vaultIdInputs = screen.getAllByTestId('vault-id-input') as HTMLInputElement[];

			// Set vault id for token2
			await userEvent.clear(vaultIdInputs[0]);
			await userEvent.type(vaultIdInputs[0], '234');

			// Set vault id for token1
			await userEvent.clear(vaultIdInputs[1]);
			await userEvent.type(vaultIdInputs[1], '123');

			// Click the "Deploy Order" button
			const deployButton = screen.getByText('Deploy Order');
			await userEvent.click(deployButton);

			await waitFor(
				async () => {
					const disclaimerButton = screen.getByText('Deploy');
					await userEvent.click(disclaimerButton);
				},
				{ timeout: 5000 }
			);

			const getDeploymentArgs = async () => {
				if (!registry) {
					throw new Error('Registry not initialized');
				}
				const builderResult = await registry.getOrderBuilder('fixed-limit', 'base');
				if (builderResult.error) {
					throw new Error(builderResult.error.readableMsg ?? builderResult.error.msg);
				}
				const builder = builderResult.value;
				await builder.setSelectToken('token1', '0x000000000000012def132e61759048be5b5c6033');
				await builder.setSelectToken('token2', '0x00000000000007c8612ba63df8ddefd9e6077c97');
				builder.setVaultId('output', 'token2', '234');
				builder.setVaultId('input', 'token1', '123');
				builder.setFieldValue('fixed-io', '10');
				const args = await builder.getDeploymentTransactionArgs(
					'0x999999cf1046e68e36E1aA2E0E07105eDDD1f08E'
				);
				return args.value;
			};
			await new Promise((resolve) => setTimeout(resolve, 10000));

			const args = await getDeploymentArgs();

			// @ts-expect-error mock is not typed
			const callArgs = handleTransactionConfirmationModal.mock.calls.at(-1)?.[0] as
				| TransactionConfirmationProps
				| undefined;

			expect(callArgs).toBeDefined();
			if (!callArgs) {
				return;
			}
			expect(callArgs.modalTitle).toEqual('Deploying your order');

			const { prefixEnd, suffixEnd } = findLockRegion(
				callArgs.args.calldata,
				args?.deploymentCalldata || ''
			);

			expect(callArgs.args.calldata.length).toEqual(args?.deploymentCalldata.length);
			expect(callArgs.args.calldata.slice(0, prefixEnd)).toEqual(
				args?.deploymentCalldata.slice(0, prefixEnd)
			);
			// The middle section of the calldata is different because of secret and nonce
			expect(callArgs.args.calldata.slice(prefixEnd, suffixEnd)).not.toEqual(
				args?.deploymentCalldata.slice(prefixEnd, suffixEnd)
			);
			expect(callArgs.args.calldata.slice(suffixEnd)).toEqual(
				args?.deploymentCalldata.slice(suffixEnd)
			);
			expect(callArgs.args.toAddress).toEqual(args?.raindexAddress);
			expect(callArgs.args.chainId).toEqual(args?.chainId);
		},
		{ timeout: 60000, retry: DEFAULT_MAX_RETRIES }
	);

	it(
		'Auction order',
		async () => {
			const auctionDeploymentDetails = registry.getDeploymentDetails('auction-dca');
			if (auctionDeploymentDetails.error) {
				throw new Error('Failed to get deployment details');
			}
			const deployment = auctionDeploymentDetails.value.get('base') as NameAndDescriptionCfg;
			const auctionOrderDetail = registry
				.getAllOrderDetails()
				.value?.valid.get('auction-dca') as NameAndDescriptionCfg;

			mockPageStore.mockSetSubscribeValue({
				data: {
					orderName: 'auction-dca',
					deployment: { key: 'base', ...deployment },
					registry,
					orderDetail: auctionOrderDetail
				}
			});

			const screen = render(Page);

			// Wait for the builder provider to be in the document
			await waitFor(
				() => {
					expect(screen.getByTestId('builder-provider')).toBeInTheDocument();
				},
				{ timeout: 30000 }
			);

			// Check that the token dropdowns are present. The auction-dca order
			// has a single output token ("output") and a single input token
			// ("input"), in that select-token order.
			await waitFor(
				() => {
					expect(screen.getAllByRole('button', { name: /chevron down solid/i }).length).toBe(2);
				},
				{ timeout: 30000 }
			);
			await selectTokenByAddress(screen, 0, TOKEN1_ADDRESS, 'select-token-success-output');
			await new Promise((resolve) => setTimeout(resolve, 2000));

			await selectTokenByAddress(screen, 1, TOKEN2_ADDRESS, 'select-token-success-input');
			// Allow async WASM operations (getTokenInfo, getAccountBalance) to
			// settle so their &self borrows are released before setFieldValue
			// takes &mut self.
			await new Promise((resolve) => setTimeout(resolve, 2000));

			let timePerAmountEpochInput!: HTMLInputElement;
			await waitFor(
				() => {
					timePerAmountEpochInput = screen.getByTestId(
						'binding-time-per-amount-epoch-input'
					) as HTMLInputElement;
					expect(timePerAmountEpochInput).toBeInTheDocument();
				},
				{ timeout: 30000 }
			);
			await userEvent.clear(timePerAmountEpochInput);
			await userEvent.type(timePerAmountEpochInput, '60');

			const amountPerEpochInput = screen.getByTestId(
				'binding-amount-per-epoch-input'
			) as HTMLInputElement;
			await userEvent.clear(amountPerEpochInput);
			await userEvent.type(amountPerEpochInput, '10');

			const maxTradeAmountInput = screen.getByTestId(
				'binding-max-trade-amount-input'
			) as HTMLInputElement;
			await userEvent.clear(maxTradeAmountInput);
			await userEvent.type(maxTradeAmountInput, '100');

			const minTradeAmountInput = screen.getByTestId(
				'binding-min-trade-amount-input'
			) as HTMLInputElement;
			await userEvent.clear(minTradeAmountInput);
			await userEvent.type(minTradeAmountInput, '1');

			const baselineInput = screen.getByTestId('binding-baseline-input') as HTMLInputElement;
			await userEvent.clear(baselineInput);
			await userEvent.type(baselineInput, '10');

			const initialIoInput = screen.getByTestId('binding-initial-io-input') as HTMLInputElement;
			await userEvent.clear(initialIoInput);
			await userEvent.type(initialIoInput, '10');

			const showAdvancedOptionsButton = screen.getByText('Show advanced options');
			await userEvent.click(showAdvancedOptionsButton);

			const vaultIdInputs = screen.getAllByTestId('vault-id-input') as HTMLInputElement[];

			// Set vault id for output
			await userEvent.clear(vaultIdInputs[0]);
			await userEvent.type(vaultIdInputs[0], '123');

			// Set vault id for input
			await userEvent.clear(vaultIdInputs[1]);
			await userEvent.type(vaultIdInputs[1], '234');

			// Click the "Deploy Order" button
			const deployButton = screen.getByText('Deploy Order');
			await userEvent.click(deployButton);

			await waitFor(
				async () => {
					const disclaimerButton = screen.getByText('Deploy');
					await userEvent.click(disclaimerButton);
				},
				{ timeout: 5000 }
			);

			const getDeploymentArgs = async () => {
				const builderResult = await registry.getOrderBuilder('auction-dca', 'base');
				if (builderResult.error) {
					throw new Error(builderResult.error.readableMsg ?? builderResult.error.msg);
				}
				const builder = builderResult.value;
				await builder.setSelectToken('output', TOKEN1_ADDRESS);
				await builder.setSelectToken('input', TOKEN2_ADDRESS);
				builder.setVaultId('output', 'output', '123');
				builder.setVaultId('input', 'input', '234');
				builder.setFieldValue('time-per-amount-epoch', '60');
				builder.setFieldValue('amount-per-epoch', '10');
				builder.setFieldValue('max-trade-amount', '100');
				builder.setFieldValue('min-trade-amount', '1');
				builder.setFieldValue('baseline', '10');
				builder.setFieldValue('initial-io', '10');
				const args = await builder.getDeploymentTransactionArgs(ACCOUNT);
				return args.value;
			};
			await new Promise((resolve) => setTimeout(resolve, 10000));

			const args = await getDeploymentArgs();

			// @ts-expect-error mock is not typed
			const callArgs = handleTransactionConfirmationModal.mock.calls.at(-1)?.[0] as
				| TransactionConfirmationProps
				| undefined;

			expect(callArgs).toBeDefined();
			if (!callArgs) {
				return;
			}
			expect(callArgs.modalTitle).toEqual('Deploying your order');

			const { prefixEnd, suffixEnd } = findLockRegion(
				callArgs.args.calldata,
				args?.deploymentCalldata || ''
			);

			expect(callArgs.args.calldata.length).toEqual(args?.deploymentCalldata.length);
			expect(callArgs.args.calldata.slice(0, prefixEnd)).toEqual(
				args?.deploymentCalldata.slice(0, prefixEnd)
			);
			// The middle section of the calldata is different because of secret and nonce
			expect(callArgs.args.calldata.slice(prefixEnd, suffixEnd)).not.toEqual(
				args?.deploymentCalldata.slice(prefixEnd, suffixEnd)
			);
			expect(callArgs.args.calldata.slice(suffixEnd)).toEqual(
				args?.deploymentCalldata.slice(suffixEnd)
			);
			expect(callArgs.args.toAddress).toEqual(args?.raindexAddress);
			expect(callArgs.args.chainId).toEqual(args?.chainId);
		},
		{ timeout: 60000, retry: DEFAULT_MAX_RETRIES }
	);

	it(
		'Dynamic spread order',
		async () => {
			const dynamicSpreadDeploymentDetails = registry.getDeploymentDetails('dynamic-spread');
			if (dynamicSpreadDeploymentDetails.error) {
				throw new Error('Failed to get deployment details');
			}
			const deployment = dynamicSpreadDeploymentDetails.value.get('base') as NameAndDescriptionCfg;
			const dynamicSpreadOrderDetail = registry
				.getAllOrderDetails()
				.value?.valid.get('dynamic-spread') as NameAndDescriptionCfg;

			mockPageStore.mockSetSubscribeValue({
				data: {
					orderName: 'dynamic-spread',
					deployment: { key: 'base', ...deployment },
					registry,
					orderDetail: dynamicSpreadOrderDetail
				}
			});

			const screen = render(Page);

			// Wait for the builder provider to be in the document
			await waitFor(
				() => {
					expect(screen.getByTestId('builder-provider')).toBeInTheDocument();
				},
				{ timeout: 30000 }
			);

			// The dynamic-spread order rotates two tokens ("token1", "token2"),
			// each used as both an input and an output.
			await waitFor(
				() => {
					expect(screen.getAllByRole('button', { name: /chevron down solid/i }).length).toBe(2);
				},
				{ timeout: 30000 }
			);
			await selectTokenByAddress(screen, 0, TOKEN1_ADDRESS, 'select-token-success-token1');
			await new Promise((resolve) => setTimeout(resolve, 2000));

			await selectTokenByAddress(screen, 1, TOKEN2_ADDRESS, 'select-token-success-token2');
			// Allow async WASM operations (getTokenInfo, getAccountBalance) to
			// settle so their &self borrows are released before setFieldValue
			// takes &mut self.
			await new Promise((resolve) => setTimeout(resolve, 2000));

			let amountIsFastExitButton!: HTMLElement;
			await waitFor(
				() => {
					amountIsFastExitButton = screen.getByTestId('binding-amount-is-fast-exit-preset-Yes');
					expect(amountIsFastExitButton).toBeInTheDocument();
				},
				{ timeout: 30000 }
			);
			await userEvent.click(amountIsFastExitButton);

			const notAmountIsFastExitButton = screen.getByTestId(
				'binding-not-amount-is-fast-exit-preset-No'
			) as HTMLElement;
			await userEvent.click(notAmountIsFastExitButton);

			const initialIoInput = screen.getByTestId('binding-initial-io-input') as HTMLInputElement;
			await userEvent.clear(initialIoInput);
			await userEvent.type(initialIoInput, '100');

			const maxAmountInput = screen.getByTestId('binding-max-amount-input') as HTMLInputElement;
			await userEvent.clear(maxAmountInput);
			await userEvent.type(maxAmountInput, '1000');

			const minAmountInput = screen.getByTestId('binding-min-amount-input') as HTMLInputElement;
			await userEvent.clear(minAmountInput);
			await userEvent.type(minAmountInput, '10');

			const showAdvancedOptionsButton = screen.getByText('Show advanced options');
			await userEvent.click(showAdvancedOptionsButton);

			// The order lists both tokens as both inputs and outputs, so there
			// are four vault id inputs: Output token1, Output token2, Input
			// token1, Input token2 (in that render order).
			const vaultIdInputs = screen.getAllByTestId('vault-id-input') as HTMLInputElement[];

			await userEvent.clear(vaultIdInputs[0]);
			await userEvent.type(vaultIdInputs[0], '123');

			await userEvent.clear(vaultIdInputs[1]);
			await userEvent.type(vaultIdInputs[1], '234');

			await userEvent.clear(vaultIdInputs[2]);
			await userEvent.type(vaultIdInputs[2], '345');

			await userEvent.clear(vaultIdInputs[3]);
			await userEvent.type(vaultIdInputs[3], '456');

			// Click the "Deploy Order" button
			const deployButton = screen.getByText('Deploy Order');
			await userEvent.click(deployButton);

			await waitFor(
				async () => {
					const disclaimerButton = screen.getByText('Deploy');
					await userEvent.click(disclaimerButton);
				},
				{ timeout: 5000 }
			);

			const getDeploymentArgs = async () => {
				const builderResult = await registry.getOrderBuilder('dynamic-spread', 'base');
				if (builderResult.error) {
					throw new Error(builderResult.error.readableMsg ?? builderResult.error.msg);
				}
				const builder = builderResult.value;
				await builder.setSelectToken('token1', TOKEN1_ADDRESS);
				await builder.setSelectToken('token2', TOKEN2_ADDRESS);
				builder.setVaultId('output', 'token1', '123');
				builder.setVaultId('output', 'token2', '234');
				builder.setVaultId('input', 'token1', '345');
				builder.setVaultId('input', 'token2', '456');
				builder.setFieldValue('amount-is-fast-exit', '1');
				builder.setFieldValue('not-amount-is-fast-exit', '0');
				builder.setFieldValue('initial-io', '100');
				builder.setFieldValue('max-amount', '1000');
				builder.setFieldValue('min-amount', '10');
				const args = await builder.getDeploymentTransactionArgs(ACCOUNT);
				return args.value;
			};
			await new Promise((resolve) => setTimeout(resolve, 10000));

			const args = await getDeploymentArgs();

			// @ts-expect-error mock is not typed
			const callArgs = handleTransactionConfirmationModal.mock.calls.at(-1)?.[0] as
				| TransactionConfirmationProps
				| undefined;

			expect(callArgs).toBeDefined();
			if (!callArgs) {
				return;
			}
			expect(callArgs.modalTitle).toEqual('Deploying your order');

			const { prefixEnd, suffixEnd } = findLockRegion(
				callArgs.args.calldata,
				args?.deploymentCalldata || ''
			);

			expect(callArgs.args.calldata.length).toEqual(args?.deploymentCalldata.length);
			expect(callArgs.args.calldata.slice(0, prefixEnd)).toEqual(
				args?.deploymentCalldata.slice(0, prefixEnd)
			);
			// The middle section of the calldata is different because of secret and nonce
			expect(callArgs.args.calldata.slice(prefixEnd, suffixEnd)).not.toEqual(
				args?.deploymentCalldata.slice(prefixEnd, suffixEnd)
			);
			expect(callArgs.args.calldata.slice(suffixEnd)).toEqual(
				args?.deploymentCalldata.slice(suffixEnd)
			);
			expect(callArgs.args.toAddress).toEqual(args?.raindexAddress);
			expect(callArgs.args.chainId).toEqual(args?.chainId);
		},
		{ timeout: 60000, retry: DEFAULT_MAX_RETRIES }
	);
});
