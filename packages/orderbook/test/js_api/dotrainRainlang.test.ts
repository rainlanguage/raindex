import assert from 'assert';
import { afterAll, beforeAll, beforeEach, describe, it } from 'vitest';
import { WasmEncodedResult, DotrainRainlang, OrderbookYaml, RaindexClient } from '../../dist/cjs';
import { getLocal } from 'mockttp';

const SPEC_VERSION = OrderbookYaml.getCurrentSpecVersion().value;

const extractWasmEncodedData = <T>(result: WasmEncodedResult<T>, errorMessage?: string): T => {
	if (result.error) {
		assert.fail(errorMessage ?? result.error.msg);
	}
	if (typeof void 0 === typeof result.value) {
		return result.value as T;
	}
	return result.value;
};

const MOCK_SETTINGS_CONTENT = `
version: ${SPEC_VERSION}
networks:
  flare:
    rpcs:
      - https://mainnet.flare.org
    chain-id: 14
    currency: ETH
  base:
    rpcs:
      - https://mainnet.base.org
    chain-id: 8453
    currency: ETH
subgraphs:
  flare: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-flare/0.8/gn
  base: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-base/0.9/gn
metaboards:
  flare: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-flare-0x893BBFB7/0.1/gn
  base: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-base-0x59401C93/0.1/gn
orderbooks:
  flare:
    address: 0xCEe8Cd002F151A536394E564b84076c41bBBcD4d
    network: flare
    subgraph: flare
    local-db-remote: remote
    deployment-block: 12345
  base:
    address: 0xd2938e7c9fe3597f78832ce780feb61945c377d7
    network: base
    subgraph: base
    local-db-remote: remote
    deployment-block: 12345
rainlangs:
  flare:
    address: 0xE3989Ea7486c0F418C764e6c511e86f6E8830FAb
    network: flare
  base:
    address: 0xC1A14cE2fd58A3A2f99deCb8eDd866204eE07f8D
    network: base
tokens:
  token1:
    address: 0x4200000000000000000000000000000000000042
    network: flare
  token2:
    address: 0x4200000000000000000000000000000000000042
    network: base
`;

const MOCK_DOTRAIN_PREFIX = `
version: ${SPEC_VERSION}
gui:
  name: Test gui
  description: Test description
  short-description: Test short description
  deployments:
    flare:
      name: Flare order name
      description: Flare order description
      deposits:
        - token: token1
          presets:
            - "0"
      fields:
        - binding: test-binding
          name: Test binding
          description: Test binding description
          presets:
            - value: "0xbeef"
          default: 10
    base:
      name: Base order name
      description: Base order description
      deposits:
        - token: token2
          presets:
            - "0"
      fields:
        - binding: test-binding
          name: Test binding
          description: Test binding description
          presets:
            - value: "0xbeef"
          default: 10
scenarios:
  flare:
    rainlang: flare
    runs: 1
  base:
    rainlang: base
    runs: 1
orders:
  flare:
    orderbook: flare
    inputs:
      - token: token1
    outputs:
      - token: token1
  base:
    orderbook: base
    inputs:
      - token: token2
    outputs:
      - token: token2
deployments:
  flare:
    scenario: flare
    order: flare
  base:
    scenario: base
    order: base`;

const FIRST_DOTRAIN_CONTENT = `
${MOCK_DOTRAIN_PREFIX}
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;`;

const SECOND_DOTRAIN_CONTENT = `
${MOCK_DOTRAIN_PREFIX}
---
#calculate-io
_ _: 1 1;
#handle-io
:;
#handle-add-order
:;`;

describe('Rain Orderbook JS API Package Bindgen Tests - Dotrain Rainlang', async function () {
	const mockServer = getLocal();
	beforeAll(async () => {
		await mockServer.start(8231);
	});
	afterAll(async () => {
		await mockServer.stop();
	});
	beforeEach(() => {
		mockServer.reset();
	});

	describe('DotrainRainlang Constructor', () => {

		it('should create rainlang and fetch all content successfully', async () => {
			const rainlangContent = `http://localhost:8231/settings.yaml
fixed-limit http://localhost:8231/fixed-limit.rain
auction-dca http://localhost:8231/auction-dca.rain`;

			await mockServer.forGet('/registry.txt').thenReply(200, rainlangContent);
			await mockServer.forGet('/settings.yaml').thenReply(200, MOCK_SETTINGS_CONTENT);
			await mockServer.forGet('/fixed-limit.rain').thenReply(200, FIRST_DOTRAIN_CONTENT);
			await mockServer.forGet('/auction-dca.rain').thenReply(200, SECOND_DOTRAIN_CONTENT);

			const result = await DotrainRainlang.new('http://localhost:8231/registry.txt');
			const registry = extractWasmEncodedData(result);

			assert.strictEqual(registry.rainlangUrl, 'http://localhost:8231/registry.txt');
			assert.strictEqual(registry.settingsUrl, 'http://localhost:8231/settings.yaml');
			assert.strictEqual(registry.rainlang, rainlangContent);
			assert.strictEqual(registry.settings, MOCK_SETTINGS_CONTENT);

			const orderUrls = registry.orderUrls;
			assert.strictEqual(orderUrls.size, 2);
			assert.strictEqual(orderUrls.get('fixed-limit'), 'http://localhost:8231/fixed-limit.rain');
			assert.strictEqual(orderUrls.get('auction-dca'), 'http://localhost:8231/auction-dca.rain');

			const orders = registry.orders;
			assert.strictEqual(orders.size, 2);
			assert(orders.has('fixed-limit'));
			assert(orders.has('auction-dca'));
		});

		it('should handle invalid rainlang format', async () => {
			const invalidContent = 'invalid format without proper structure';
			await mockServer.forGet('/invalid.txt').thenReply(200, invalidContent);

			const result = await DotrainRainlang.new('http://localhost:8231/invalid.txt');
			assert(result.error);
		});

		it('should handle empty rainlang file', async () => {
			await mockServer.forGet('/empty.txt').thenReply(200, '');

			const result = await DotrainRainlang.new('http://localhost:8231/empty.txt');
			assert(result.error);
			assert(result.error.readableMsg.includes('Invalid rainlang format'));
		});

		it('should handle settings fetch error', async () => {
			const rainlangContent =
				'http://localhost:8231/nonexistent-settings.yaml\norder1 http://localhost:8231/order1.rain';
			await mockServer.forGet('/registry.txt').thenReply(200, rainlangContent);
			await mockServer.forGet('/nonexistent-settings.yaml').thenReply(404);

			const result = await DotrainRainlang.new('http://localhost:8231/registry.txt');
			assert(result.error);
		});

		it('should validate rainlang format without fetching orders', async () => {
			const rainlangContent = `http://localhost:8231/settings.yaml
fixed-limit http://localhost:8231/fixed-limit.rain`;

			await mockServer.forGet('/registry.txt').thenReply(200, rainlangContent);

			const result = await DotrainRainlang.validate('http://localhost:8231/registry.txt');
			const value = extractWasmEncodedData(result);
			assert.strictEqual(value, undefined);
		});

		it('should fail validation for invalid rainlang format', async () => {
			await mockServer.forGet('/invalid-registry.txt').thenReply(200, 'invalid');

			const result = await DotrainRainlang.validate('http://localhost:8231/invalid-registry.txt');
			assert(result.error);
		});
	});

	describe('DotrainRainlang Order Management', () => {
		let registry: DotrainRainlang;

		beforeEach(async () => {
			const rainlangContent = `http://localhost:8231/settings.yaml
fixed-limit http://localhost:8231/fixed-limit.rain
auction-dca http://localhost:8231/auction-dca.rain`;

			await mockServer.forGet('/registry.txt').thenReply(200, rainlangContent);
			await mockServer.forGet('/settings.yaml').thenReply(200, MOCK_SETTINGS_CONTENT);
			await mockServer.forGet('/fixed-limit.rain').thenReply(200, FIRST_DOTRAIN_CONTENT);
			await mockServer.forGet('/auction-dca.rain').thenReply(200, SECOND_DOTRAIN_CONTENT);

			const result = await DotrainRainlang.new('http://localhost:8231/registry.txt');
			registry = extractWasmEncodedData(result);
		});

		it('should get order keys', () => {
			const keys = extractWasmEncodedData(registry.getOrderKeys());

			assert.strictEqual(keys.length, 2);
			assert(keys.includes('fixed-limit'));
			assert(keys.includes('auction-dca'));
		});

		it('should get all order details', () => {
			const orderDetails = extractWasmEncodedData(registry.getAllOrderDetails());

			assert.strictEqual(orderDetails.valid.size, 2);
			assert.strictEqual(orderDetails.invalid.size, 0);
			assert(orderDetails.valid.has('fixed-limit'));
			assert(orderDetails.valid.has('auction-dca'));

			const fixedLimitDetails = orderDetails.valid.get('fixed-limit');
			assert(fixedLimitDetails);
			assert.strictEqual(fixedLimitDetails.name, 'Test gui');
			assert.strictEqual(fixedLimitDetails.description, 'Test description');
			assert.strictEqual(fixedLimitDetails.short_description, 'Test short description');
		});

		it('should handle mixed valid and invalid orders', async () => {
			mockServer.reset();

			const rainlangContent = `http://localhost:8231/settings.yaml
valid-order http://localhost:8231/valid.rain
invalid-order http://localhost:8231/invalid.rain`;

			await mockServer.forGet('/registry.txt').thenReply(200, rainlangContent);
			await mockServer.forGet('/settings.yaml').thenReply(200, MOCK_SETTINGS_CONTENT);
			await mockServer.forGet('/valid.rain').thenReply(200, FIRST_DOTRAIN_CONTENT);
			await mockServer.forGet('/invalid.rain').thenReply(200, 'not a dotrain file');

			const rainlangResult = await DotrainRainlang.new('http://localhost:8231/registry.txt');
			const mixedRainlang = extractWasmEncodedData(rainlangResult);

			const orderDetails = extractWasmEncodedData(mixedRainlang.getAllOrderDetails());

			assert.strictEqual(orderDetails.valid.size, 1);
			assert.strictEqual(orderDetails.invalid.size, 1);
			assert(orderDetails.valid.has('valid-order'));
			assert(orderDetails.invalid.has('invalid-order'));
		});

		it('should get deployment details for specific order', () => {
			const deploymentDetails = extractWasmEncodedData(
				registry.getDeploymentDetails('fixed-limit')
			);

			assert.strictEqual(deploymentDetails.size, 2);
			assert(deploymentDetails.has('flare'));
			assert(deploymentDetails.has('base'));

			const flareDetails = deploymentDetails.get('flare');
			assert(flareDetails);
			assert.strictEqual(flareDetails.name, 'Flare order name');
			assert.strictEqual(flareDetails.description, 'Flare order description');

			const baseDetails = deploymentDetails.get('base');
			assert(baseDetails);
			assert.strictEqual(baseDetails.name, 'Base order name');
			assert.strictEqual(baseDetails.description, 'Base order description');
		});

		it('should handle deployment details for non-existent order', () => {
			const result = registry.getDeploymentDetails('non-existent');
			assert(result.error);
			assert(result.error.readableMsg.includes("order key 'non-existent' was not found"));
		});
	});

	describe('DotrainRainlang GUI Creation', () => {
		let registry: DotrainRainlang;

		beforeEach(async () => {
			const rainlangContent = `http://localhost:8231/settings.yaml
fixed-limit http://localhost:8231/fixed-limit.rain`;

			await mockServer.forGet('/registry.txt').thenReply(200, rainlangContent);
			await mockServer.forGet('/settings.yaml').thenReply(200, MOCK_SETTINGS_CONTENT);
			await mockServer.forGet('/fixed-limit.rain').thenReply(200, FIRST_DOTRAIN_CONTENT);

			registry = extractWasmEncodedData(
				await DotrainRainlang.new('http://localhost:8231/registry.txt')
			);
		});

		it('should create GUI for valid order and deployment', async () => {
			const gui = extractWasmEncodedData(await registry.getGui('fixed-limit', 'flare', null, null));

			const currentDeployment = extractWasmEncodedData(gui.getCurrentDeployment());

			assert.strictEqual(currentDeployment.name, 'Flare order name');
			assert.strictEqual(currentDeployment.description, 'Flare order description');
		});

		it('should create GUI with state update callback', async () => {
			const stateCallback = () => {};

			const gui = extractWasmEncodedData(
				await registry.getGui('fixed-limit', 'base', null, stateCallback)
			);

			const currentDeployment = extractWasmEncodedData(gui.getCurrentDeployment());

			assert.strictEqual(currentDeployment.name, 'Base order name');
			assert.strictEqual(currentDeployment.description, 'Base order description');
		});

		it('should restore GUI from serialized state when provided', async () => {
			let gui = extractWasmEncodedData(await registry.getGui('fixed-limit', 'flare', null, null));

			gui.setFieldValue('test-binding', '42');
			const serializedState = extractWasmEncodedData<string>(gui.serializeState());

			gui = extractWasmEncodedData(
				await registry.getGui('fixed-limit', 'flare', serializedState, null)
			);

			const fieldValue = extractWasmEncodedData<{ value: string }>(
				gui.getFieldValue('test-binding')
			);

			assert.strictEqual(fieldValue.value, '42');
		});

		it('should handle GUI creation for non-existent order', async () => {
			const result = await registry.getGui('non-existent', 'flare', null, null);
			assert(result.error);
			assert(result.error.readableMsg.includes("order key 'non-existent' was not found"));
		});
	});

	const MOCK_SETTINGS_WITH_TOKENS = `
version: ${SPEC_VERSION}
networks:
  mainnet:
    rpcs:
      - https://mainnet.infura.io
    chain-id: 1
    currency: ETH
tokens:
  weth:
    network: mainnet
    address: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
    decimals: 18
    label: Wrapped Ether
    symbol: WETH
  usdc:
    network: mainnet
    address: 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
    decimals: 6
    label: USD Coin
    symbol: USDC
orderbooks:
  mainnet:
    address: 0x1234567890123456789012345678901234567890
    network: mainnet
rainlangs:
  mainnet:
    address: 0x1234567890123456789012345678901234567890
    network: mainnet
`;

	const MOCK_DOTRAIN_SIMPLE = `
gui:
  name: Test Order
  description: Test description
  deployments:
    mainnet:
      name: Mainnet Order
      description: Mainnet deployment
      deposits:
        - token: weth
          presets:
            - "0"
      fields:
        - binding: test-binding
          name: Test binding
          presets:
            - value: "0xbeef"
scenarios:
  mainnet:
    rainlang: mainnet
    runs: 1
orders:
  mainnet:
    orderbook: mainnet
    inputs:
      - token: weth
    outputs:
      - token: usdc
deployments:
  mainnet:
    scenario: mainnet
    order: mainnet
---
#calculate-io
_ _: 0 0;
#handle-io
:;
`;

	describe('DotrainRainlang getOrderbookYaml', () => {

		it('should return OrderbookYaml instance from settings', async () => {
			const rainlangContent = `http://localhost:8231/settings.yaml
test-order http://localhost:8231/order.rain`;

			await mockServer.forGet('/registry.txt').thenReply(200, rainlangContent);
			await mockServer.forGet('/settings.yaml').thenReply(200, MOCK_SETTINGS_WITH_TOKENS);
			await mockServer.forGet('/order.rain').thenReply(200, MOCK_DOTRAIN_SIMPLE);

			const registry = extractWasmEncodedData(
				await DotrainRainlang.new('http://localhost:8231/registry.txt')
			);

			const orderbookYamlResult = registry.getOrderbookYaml();
			const orderbookYaml = extractWasmEncodedData(orderbookYamlResult);

			assert.ok(orderbookYaml, 'OrderbookYaml instance should be returned');
			assert.strictEqual(typeof orderbookYaml.getTokens, 'function');
			assert.strictEqual(typeof orderbookYaml.getOrderbookByAddress, 'function');
		});
	});

	describe('DotrainRainlang getRaindexClient', () => {
		it('should return RaindexClient instance from settings', async () => {
			const rainlangContent = `http://localhost:8231/settings.yaml
test-order http://localhost:8231/order.rain`;

			await mockServer.forGet('/registry.txt').thenReply(200, rainlangContent);
			await mockServer.forGet('/settings.yaml').thenReply(200, MOCK_SETTINGS_WITH_TOKENS);
			await mockServer.forGet('/order.rain').thenReply(200, MOCK_DOTRAIN_SIMPLE);

			const registry = extractWasmEncodedData(
				await DotrainRainlang.new('http://localhost:8231/registry.txt')
			);

			const raindexClientResult = await registry.getRaindexClient();
			const raindexClient = extractWasmEncodedData(raindexClientResult);

			assert.ok(raindexClient, 'RaindexClient instance should be returned');
		});
	});
});
