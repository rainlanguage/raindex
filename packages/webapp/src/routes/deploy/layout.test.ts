import { describe, it, expect, vi, beforeEach } from 'vitest';
import { load } from './+layout';

describe('deploy /+layout load function', () => {
	const mockGetAllOrderDetails = vi.fn();
	const mockOrdersGet = vi.fn();

	const makeRegistry = () => ({
		getAllOrderDetails: mockGetAllOrderDetails,
		orders: { get: mockOrdersGet }
	});

	const mockParent = vi.fn();

	beforeEach(() => {
		vi.resetAllMocks();
	});

	const callLoad = () =>
		load({
			parent: mockParent
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		} as any);

	it('returns "Registry not loaded" when parent registry is null', async () => {
		mockParent.mockResolvedValue({ registry: null });

		const result = await callLoad();

		expect(result).toEqual({
			validOrders: [],
			invalidOrders: [],
			registry: null,
			error: 'Registry not loaded'
		});
		// getAllOrderDetails must not be reached when registry is missing
		expect(mockGetAllOrderDetails).not.toHaveBeenCalled();
	});

	it('returns "Registry not loaded" when parent does not provide a registry', async () => {
		// registry is undefined -> `?? null` coalescing path
		mockParent.mockResolvedValue({});

		const result = await callLoad();

		expect(result.registry).toBeNull();
		expect(result.error).toBe('Registry not loaded');
	});

	it('returns the registry error readableMsg when getAllOrderDetails errors', async () => {
		const registry = makeRegistry();
		mockGetAllOrderDetails.mockReturnValue({
			error: { readableMsg: 'Human readable failure', msg: 'raw failure' }
		});
		mockParent.mockResolvedValue({ registry });

		const result = await callLoad();

		expect(result).toEqual({
			validOrders: [],
			invalidOrders: [],
			registry,
			error: 'Human readable failure'
		});
	});

	it('falls back to error.msg when readableMsg is absent on the registry error', async () => {
		const registry = makeRegistry();
		mockGetAllOrderDetails.mockReturnValue({
			error: { msg: 'raw failure only' }
		});
		mockParent.mockResolvedValue({ registry });

		const result = await callLoad();

		expect(result.error).toBe('raw failure only');
		expect(result.validOrders).toEqual([]);
		expect(result.invalidOrders).toEqual([]);
	});

	it('does not treat a falsy-but-present error field as an error', async () => {
		// `if (orderDetails.error)` must be falsy here so we proceed to mapping,
		// not return early with `undefined` as the error message.
		const registry = makeRegistry();
		mockGetAllOrderDetails.mockReturnValue({
			error: null,
			value: { valid: new Map(), invalid: new Map() }
		});
		mockParent.mockResolvedValue({ registry });

		const result = await callLoad();

		expect(result.error).toBeNull();
		expect(result.validOrders).toEqual([]);
		expect(result.invalidOrders).toEqual([]);
	});

	it('maps valid orders with name, details and dotrain from registry.orders', async () => {
		const registry = makeRegistry();
		const detailsA = { name: 'Order A', description: 'desc A' };
		const detailsB = { name: 'Order B', description: 'desc B' };
		mockGetAllOrderDetails.mockReturnValue({
			value: {
				valid: new Map([
					['order-a', detailsA],
					['order-b', detailsB]
				]),
				invalid: new Map()
			}
		});
		mockOrdersGet.mockImplementation((name: string) =>
			name === 'order-a' ? 'dotrain-a-source' : 'dotrain-b-source'
		);
		mockParent.mockResolvedValue({ registry });

		const result = await callLoad();

		expect(result.error).toBeNull();
		expect(result.validOrders).toEqual([
			{ name: 'order-a', dotrain: 'dotrain-a-source', details: detailsA },
			{ name: 'order-b', dotrain: 'dotrain-b-source', details: detailsB }
		]);
		// dotrain must come from the looked-up order keyed by its name
		expect(mockOrdersGet).toHaveBeenCalledWith('order-a');
		expect(mockOrdersGet).toHaveBeenCalledWith('order-b');
	});

	it('falls back to empty-string dotrain when registry.orders has no entry', async () => {
		const registry = makeRegistry();
		const details = { name: 'Order A', description: 'desc A' };
		mockGetAllOrderDetails.mockReturnValue({
			value: {
				valid: new Map([['order-a', details]]),
				invalid: new Map()
			}
		});
		// registry.orders.get returns undefined -> `?? ''`
		mockOrdersGet.mockReturnValue(undefined);
		mockParent.mockResolvedValue({ registry });

		const result = await callLoad();

		expect(result.validOrders).toEqual([{ name: 'order-a', dotrain: '', details }]);
	});

	it('maps invalid orders using error.readableMsg', async () => {
		const registry = makeRegistry();
		mockGetAllOrderDetails.mockReturnValue({
			value: {
				valid: new Map(),
				invalid: new Map([['broken-order', { readableMsg: 'nice message', msg: 'raw' }]])
			}
		});
		mockParent.mockResolvedValue({ registry });

		const result = await callLoad();

		expect(result.invalidOrders).toEqual([{ name: 'broken-order', error: 'nice message' }]);
		expect(result.error).toBeNull();
	});

	it('falls back to error.msg for invalid orders when readableMsg is absent', async () => {
		const registry = makeRegistry();
		mockGetAllOrderDetails.mockReturnValue({
			value: {
				valid: new Map(),
				invalid: new Map([['broken-order', { msg: 'raw message only' }]])
			}
		});
		mockParent.mockResolvedValue({ registry });

		const result = await callLoad();

		expect(result.invalidOrders).toEqual([{ name: 'broken-order', error: 'raw message only' }]);
	});

	it('returns both valid and invalid orders with error null on success', async () => {
		const registry = makeRegistry();
		const validDetails = { name: 'Good', description: 'good desc' };
		mockGetAllOrderDetails.mockReturnValue({
			value: {
				valid: new Map([['good-order', validDetails]]),
				invalid: new Map([['bad-order', { readableMsg: 'bad reason', msg: 'raw' }]])
			}
		});
		mockOrdersGet.mockReturnValue('good-dotrain');
		mockParent.mockResolvedValue({ registry });

		const result = await callLoad();

		expect(result).toEqual({
			validOrders: [{ name: 'good-order', dotrain: 'good-dotrain', details: validDetails }],
			invalidOrders: [{ name: 'bad-order', error: 'bad reason' }],
			registry,
			error: null
		});
	});

	it('catches a thrown Error and returns its message', async () => {
		const registry = makeRegistry();
		mockGetAllOrderDetails.mockImplementation(() => {
			throw new Error('boom from registry');
		});
		mockParent.mockResolvedValue({ registry });

		const result = await callLoad();

		expect(result).toEqual({
			validOrders: [],
			invalidOrders: [],
			registry,
			error: 'boom from registry'
		});
	});

	it('catches a non-Error throw and returns the generic message', async () => {
		const registry = makeRegistry();
		// throw a non-Error value -> `error instanceof Error` is false
		mockGetAllOrderDetails.mockImplementation(() => {
			throw 'a plain string, not an Error';
		});
		mockParent.mockResolvedValue({ registry });

		const result = await callLoad();

		expect(result.error).toBe('Unknown error occurred');
		expect(result.validOrders).toEqual([]);
		expect(result.invalidOrders).toEqual([]);
		expect(result.registry).toBe(registry);
	});
});
