import assert from 'assert';
import { afterAll, beforeAll, beforeEach, describe, expect, it } from 'vitest';
import { OrderbookYaml, OrderbookCfg, WasmEncodedResult } from '../../dist/cjs';
import { getLocal } from 'mockttp';

const SPEC_VERSION = OrderbookYaml.getCurrentSpecVersion().value;

const YAML_WITHOUT_ORDERBOOK = `
version: ${SPEC_VERSION}

networks:
    some-network:
        rpcs:
            - http://localhost:8085/rpc-url
        chain-id: 123
        network-id: 123
        currency: ETH

subgraphs:
    some-sg: https://www.some-sg.com

metaboards:
    test: https://metaboard.com

rainlangs:
    some-rainlang:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba

local-db-remotes:
  some-orderbook: http://example.com

local-db-sync:
  some-orderbook:
    batch-size: 2000
    max-concurrent-batches: 10
    retry-attempts: 3
    retry-delay-ms: 250
    rate-limit-delay-ms: 1000
    finality-depth: 30
    bootstrap-block-threshold: 1000
    sync-interval-ms: 5000

orderbooks:
    some-orderbook:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: some-network
        subgraph: some-sg
        deployment-block: 12345

tokens:
    token1:
        network: some-network
        address: 0xc2132d05d31c914a87c6611c10748aeb04b58e8f
        decimals: 6
        label: Token 1
        symbol: T1
    token2:
        network: some-network
        address: 0x8f3cf7ad23cd3cadbd9735aff958023239c6a063
        decimals: 18
        label: Token 2
        symbol: T2

scenarios:
    some-scenario:
        rainlang: some-rainlang
        bindings:
            test-binding: 5
        scenarios:
            sub-scenario:
                bindings:
                    another-binding: 300

orders:
    some-order:
      inputs:
        - token: token1
          vault-id: 1
      outputs:
        - token: token2
          vault-id: 1
      rainlang: some-rainlang
      orderbook: some-orderbook

deployments:
    some-deployment:
        scenario: some-scenario
        order: some-order
    other-deployment:
        scenario: some-scenario.sub-scenario
        order: some-order
`;

const extractWasmEncodedData = <T>(result: WasmEncodedResult<T>, errorMessage?: string): T => {
	if (result.error) {
		assert.fail(errorMessage ?? result.error.msg);
	}

	if (typeof void 0 === typeof result.value) {
		return result.value as T;
	}

	return result.value;
};

const buildYaml = (source: string, validate?: boolean): OrderbookYaml => {
	const result = OrderbookYaml.new([source], validate);
	return extractWasmEncodedData<OrderbookYaml>(result);
};

describe('Rain Orderbook JS API Package Bindgen Tests - Settings', async function () {
	it('should return current spec version', async function () {
		const result = OrderbookYaml.getCurrentSpecVersion();
		if (result.error) assert.fail(result.error.msg);
		const version = result.value;
		assert.ok(version);
		assert.strictEqual(typeof version, 'string');
		assert.ok(version.length > 0);
	});

	it('should create a new settings object', async function () {
		const orderbookYaml = buildYaml(YAML_WITHOUT_ORDERBOOK);
		assert.ok(orderbookYaml);
	});

	describe('orderbook tests', async function () {
		it('should get the orderbook', async function () {
			const orderbookYaml = buildYaml(YAML_WITHOUT_ORDERBOOK);

			const orderbook = extractWasmEncodedData<OrderbookCfg>(
				orderbookYaml.getOrderbookByAddress('0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6')
			);
			assert.equal(orderbook.address, '0xc95a5f8efe14d7a20bd2e5bafec4e71f8ce0b9a6');
			assert.equal(orderbook.network.chainId, 123);
			assert.equal(orderbook.subgraph.url, 'https://www.some-sg.com/');

			let result = orderbookYaml.getOrderbookByAddress('invalid-address');
			if (!result.error) expect.fail('Expected error');
			expect(result.error.msg).toBe('Invalid address: odd number of digits');
			expect(result.error.readableMsg).toBe(
				'The provided address is invalid. Please ensure the address is in the correct hexadecimal format. Error: "odd number of digits"'
			);

			result = orderbookYaml.getOrderbookByAddress('0x0000000000000000000000000000000000000000');
			if (!result.error) expect.fail('Expected error');
			expect(result.error.msg).toBe(
				'Orderbook yaml error: orderbook with address: 0x0000000000000000000000000000000000000000 not found'
			);
			expect(result.error.readableMsg).toBe(
				'There was an error processing the YAML configuration. Please check the YAML file for any issues. Error: "orderbook with address: 0x0000000000000000000000000000000000000000 not found"'
			);
		});
	});

	describe('validation tests', async function () {
		const INVALID_YAML = `
version: ${SPEC_VERSION}

networks:
    some-network:
        rpc: http://localhost:8085/rpc-url
        chain-id: 123
        network-id: 123
        currency: ETH

orderbooks:
    some-orderbook:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: nonexistent-network
        subgraph: nonexistent-subgraph
        deployment-block: 12345
`;

		it('should succeed with valid YAML and validation enabled', async function () {
			const orderbookYaml = buildYaml(YAML_WITHOUT_ORDERBOOK, true);
			assert.ok(orderbookYaml);
		});

		it('should succeed with valid YAML and validation disabled', async function () {
			const orderbookYaml = buildYaml(YAML_WITHOUT_ORDERBOOK, false);
			assert.ok(orderbookYaml);
		});

		it('should succeed with valid YAML and default validation', async function () {
			const orderbookYaml = buildYaml(YAML_WITHOUT_ORDERBOOK);
			assert.ok(orderbookYaml);
		});

		it('should fail with invalid YAML and validation enabled', async function () {
			const result = OrderbookYaml.new([INVALID_YAML], true);
			if (!result.error) expect.fail('Expected validation error with invalid YAML');
			expect(result.error.msg).toContain('Orderbook yaml error');
			expect(result.error.readableMsg).toContain(
				'There was an error processing the YAML configuration'
			);
		});

		it('should succeed construction but fail usage with invalid YAML when validation is disabled', async function () {
			const orderbookYaml = buildYaml(INVALID_YAML, false);
			const orderbookResult = orderbookYaml.getOrderbookByAddress(
				'0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6'
			);
			if (!orderbookResult.error) expect.fail('Expected error when using invalid YAML');
			expect(orderbookResult.error.msg).toContain('Orderbook yaml error');
		});
	});

	describe('getTokens tests', async function () {
		const mockServer = getLocal();

		beforeAll(async () => {
			await mockServer.start(8087);
		});

		afterAll(async () => {
			await mockServer.stop();
		});

		beforeEach(async () => {
			await mockServer.reset();
		});

		it('should return local tokens with chainId', async function () {
			const orderbookYaml = buildYaml(YAML_WITHOUT_ORDERBOOK);
			const tokensResult = await orderbookYaml.getTokens();
			const tokens = extractWasmEncodedData(tokensResult);

			assert.strictEqual(tokens.length, 2);

			const token1 = tokens.find((t: { key: string }) => t.key === 'token1');
			assert.ok(token1, 'token1 should exist');
			assert.strictEqual(token1.chainId, 123);
			assert.strictEqual(token1.decimals, 6);
			assert.strictEqual(token1.symbol, 'T1');
			assert.strictEqual(token1.name, 'Token 1');
			assert.strictEqual(
				token1.address.toLowerCase(),
				'0xc2132d05d31c914a87c6611c10748aeb04b58e8f'
			);

			const token2 = tokens.find((t: { key: string }) => t.key === 'token2');
			assert.ok(token2, 'token2 should exist');
			assert.strictEqual(token2.chainId, 123);
			assert.strictEqual(token2.decimals, 18);
			assert.strictEqual(token2.symbol, 'T2');
			assert.strictEqual(token2.name, 'Token 2');
		});

		it('should try to fetch missing token fields from RPC and return error on failure', async function () {
			const YAML_MISSING_FIELDS = `
version: ${SPEC_VERSION}
networks:
    some-network:
        rpcs:
            - http://localhost:8087/rpc-url
        chain-id: 123
tokens:
    incomplete:
        network: some-network
        address: 0xc2132d05d31c914a87c6611c10748aeb04b58e8f
`;
			// No mock set up - RPC call will fail
			// This tests that when token fields are missing, the code attempts to fetch from RPC
			// (rather than immediately returning a "missing field" error)
			const orderbookYaml = buildYaml(YAML_MISSING_FIELDS);
			const tokensResult = await orderbookYaml.getTokens();

			assert.ok(tokensResult.error, 'Expected error when RPC fetch fails');
			expect(tokensResult.error.readableMsg).toContain('Failed to fetch token information');
		});

		it('should return tokens with correct chainId for multiple networks', async function () {
			const MULTI_NETWORK_YAML = `
version: ${SPEC_VERSION}
networks:
    mainnet:
        rpcs:
            - http://localhost:8085/rpc-url
        chain-id: 1
    polygon:
        rpcs:
            - http://localhost:8086/rpc-url
        chain-id: 137
tokens:
    weth:
        network: mainnet
        address: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
        decimals: 18
        label: Wrapped Ether
        symbol: WETH
    usdc-polygon:
        network: polygon
        address: 0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174
        decimals: 6
        label: USD Coin PoS
        symbol: USDC
`;
			const orderbookYaml = buildYaml(MULTI_NETWORK_YAML);
			const tokensResult = await orderbookYaml.getTokens();
			const tokens = extractWasmEncodedData(tokensResult);

			assert.strictEqual(tokens.length, 2);

			const mainnetToken = tokens.find((t: { chainId: number }) => t.chainId === 1);
			assert.ok(mainnetToken, 'mainnet token should exist');
			assert.strictEqual(mainnetToken.symbol, 'WETH');
			assert.strictEqual(mainnetToken.decimals, 18);

			const polygonToken = tokens.find((t: { chainId: number }) => t.chainId === 137);
			assert.ok(polygonToken, 'polygon token should exist');
			assert.strictEqual(polygonToken.symbol, 'USDC');
			assert.strictEqual(polygonToken.decimals, 6);
		});
	});

	describe('getTokens with using-tokens-from tests', async function () {
		const mockServer = getLocal();

		beforeAll(async () => {
			await mockServer.start(8232);
		});

		afterAll(async () => {
			await mockServer.stop();
		});

		beforeEach(async () => {
			await mockServer.reset();
		});

		it('should fetch and return remote tokens from using-tokens-from', async function () {
			const remoteTokensResponse = {
				name: 'Remote Tokens',
				timestamp: '2021-01-01T00:00:00.000Z',
				keywords: [],
				version: { major: 1, minor: 0, patch: 0 },
				tokens: [
					{
						chainId: 1,
						address: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
						name: 'USD Coin',
						symbol: 'USDC',
						decimals: 6
					},
					{
						chainId: 1,
						address: '0xdAC17F958D2ee523a2206206994597C13D831ec7',
						name: 'Tether USD',
						symbol: 'USDT',
						decimals: 6
					}
				],
				logoURI: 'http://localhost.com'
			};

			await mockServer.forGet('/tokens.json').thenJson(200, remoteTokensResponse);

			const yaml = `
version: ${SPEC_VERSION}
networks:
    mainnet:
        rpcs:
            - http://localhost:8085/rpc-url
        chain-id: 1
using-tokens-from:
    - http://localhost:8232/tokens.json
`;
			const orderbookYaml = buildYaml(yaml);
			const tokensResult = await orderbookYaml.getTokens();
			const tokens = extractWasmEncodedData(tokensResult);

			assert.strictEqual(tokens.length, 2);

			const usdc = tokens.find((t: { symbol: string }) => t.symbol === 'USDC');
			assert.ok(usdc, 'USDC should exist');
			assert.strictEqual(usdc.decimals, 6);
			assert.strictEqual(usdc.chainId, 1);
			assert.strictEqual(usdc.name, 'USD Coin');

			const usdt = tokens.find((t: { symbol: string }) => t.symbol === 'USDT');
			assert.ok(usdt, 'USDT should exist');
			assert.strictEqual(usdt.decimals, 6);
			assert.strictEqual(usdt.chainId, 1);
		});

		it('should return both local and remote tokens', async function () {
			const remoteTokensResponse = {
				name: 'Remote',
				timestamp: '2021-01-01T00:00:00.000Z',
				keywords: [],
				version: { major: 1, minor: 0, patch: 0 },
				tokens: [
					{
						chainId: 1,
						address: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
						name: 'USD Coin',
						symbol: 'USDC',
						decimals: 6
					}
				],
				logoURI: 'http://localhost.com'
			};

			await mockServer.forGet('/tokens.json').thenJson(200, remoteTokensResponse);

			const yaml = `
version: ${SPEC_VERSION}
networks:
    mainnet:
        rpcs:
            - http://localhost:8085/rpc-url
        chain-id: 1
using-tokens-from:
    - http://localhost:8232/tokens.json
tokens:
    weth:
        network: mainnet
        address: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
        decimals: 18
        label: Wrapped Ether
        symbol: WETH
`;
			const orderbookYaml = buildYaml(yaml);
			const tokensResult = await orderbookYaml.getTokens();
			const tokens = extractWasmEncodedData(tokensResult);

			assert.strictEqual(tokens.length, 2);
			assert.ok(
				tokens.find((t: { symbol: string }) => t.symbol === 'WETH'),
				'WETH should exist'
			);
			assert.ok(
				tokens.find((t: { symbol: string }) => t.symbol === 'USDC'),
				'USDC should exist'
			);
		});

		it('should fetch tokens from multiple using-tokens-from URLs', async function () {
			const response1 = {
				name: 'Source 1',
				timestamp: '2021-01-01T00:00:00.000Z',
				keywords: [],
				version: { major: 1, minor: 0, patch: 0 },
				tokens: [
					{
						chainId: 1,
						address: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
						name: 'USDC',
						symbol: 'USDC',
						decimals: 6
					}
				],
				logoURI: 'http://localhost.com'
			};
			const response2 = {
				name: 'Source 2',
				timestamp: '2021-01-01T00:00:00.000Z',
				keywords: [],
				version: { major: 1, minor: 0, patch: 0 },
				tokens: [
					{
						chainId: 1,
						address: '0xdAC17F958D2ee523a2206206994597C13D831ec7',
						name: 'USDT',
						symbol: 'USDT',
						decimals: 6
					}
				],
				logoURI: 'http://localhost.com'
			};

			await mockServer.forGet('/tokens1.json').thenJson(200, response1);
			await mockServer.forGet('/tokens2.json').thenJson(200, response2);

			const yaml = `
version: ${SPEC_VERSION}
networks:
    mainnet:
        rpcs:
            - http://localhost:8085/rpc-url
        chain-id: 1
using-tokens-from:
    - http://localhost:8232/tokens1.json
    - http://localhost:8232/tokens2.json
`;
			const orderbookYaml = buildYaml(yaml);
			const tokensResult = await orderbookYaml.getTokens();
			const tokens = extractWasmEncodedData(tokensResult);

			assert.strictEqual(tokens.length, 2);
			assert.ok(
				tokens.find((t: { symbol: string }) => t.symbol === 'USDC'),
				'USDC should exist'
			);
			assert.ok(
				tokens.find((t: { symbol: string }) => t.symbol === 'USDT'),
				'USDT should exist'
			);
		});

		it('should preserve extensions from remote tokens', async function () {
			const remoteTokensResponse = {
				name: 'Remote Tokens',
				timestamp: '2021-01-01T00:00:00.000Z',
				keywords: [],
				version: { major: 1, minor: 0, patch: 0 },
				tokens: [
					{
						chainId: 1,
						address: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
						name: 'USD Coin',
						symbol: 'USDC',
						decimals: 6,
						extensions: {
							isStablecoin: true,
							coingeckoId: 'usd-coin'
						}
					},
					{
						chainId: 1,
						address: '0xdAC17F958D2ee523a2206206994597C13D831ec7',
						name: 'Tether USD',
						symbol: 'USDT',
						decimals: 6
					}
				],
				logoURI: 'http://localhost.com'
			};

			await mockServer.forGet('/tokens-ext.json').thenJson(200, remoteTokensResponse);

			const yaml = `
version: ${SPEC_VERSION}
networks:
    mainnet:
        rpcs:
            - http://localhost:8085/rpc-url
        chain-id: 1
using-tokens-from:
    - http://localhost:8232/tokens-ext.json
`;
			const orderbookYaml = buildYaml(yaml);
			const tokensResult = await orderbookYaml.getTokens();
			const tokens = extractWasmEncodedData(tokensResult);

			const usdc = tokens.find((t: { symbol: string }) => t.symbol === 'USDC');
			assert.ok(usdc, 'USDC should exist');
			assert.ok(usdc.extensions instanceof Map, 'USDC extensions should be a Map');
			assert.strictEqual(usdc.extensions.get('isStablecoin'), true);
			assert.strictEqual(usdc.extensions.get('coingeckoId'), 'usd-coin');

			const usdt = tokens.find((t: { symbol: string }) => t.symbol === 'USDT');
			assert.ok(usdt, 'USDT should exist');
			assert.strictEqual(usdt.extensions, undefined);
		});

		it('should return tokens with correct chainId from multiple networks', async function () {
			const remoteTokensResponse = {
				name: 'Multi-chain Tokens',
				timestamp: '2021-01-01T00:00:00.000Z',
				keywords: [],
				version: { major: 1, minor: 0, patch: 0 },
				tokens: [
					{
						chainId: 1,
						address: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
						name: 'USD Coin',
						symbol: 'USDC',
						decimals: 6
					},
					{
						chainId: 137,
						address: '0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174',
						name: 'USD Coin (PoS)',
						symbol: 'USDC.e',
						decimals: 6
					}
				],
				logoURI: 'http://localhost.com'
			};

			await mockServer.forGet('/tokens.json').thenJson(200, remoteTokensResponse);

			const yaml = `
version: ${SPEC_VERSION}
networks:
    mainnet:
        rpcs:
            - http://localhost:8085/rpc-url
        chain-id: 1
    polygon:
        rpcs:
            - http://localhost:8086/rpc-url
        chain-id: 137
using-tokens-from:
    - http://localhost:8232/tokens.json
`;
			const orderbookYaml = buildYaml(yaml);
			const tokensResult = await orderbookYaml.getTokens();
			const tokens = extractWasmEncodedData(tokensResult);

			assert.strictEqual(tokens.length, 2);

			const mainnetUsdc = tokens.find((t: { chainId: number }) => t.chainId === 1);
			assert.ok(mainnetUsdc, 'mainnet token should exist');
			assert.strictEqual(mainnetUsdc.chainId, 1);

			const polygonUsdc = tokens.find((t: { chainId: number }) => t.chainId === 137);
			assert.ok(polygonUsdc, 'polygon token should exist');
			assert.strictEqual(polygonUsdc.chainId, 137);
		});
	});
});
