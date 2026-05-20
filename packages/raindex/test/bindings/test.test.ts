import assert from 'assert';
import { describe, it } from 'vitest';
import { getOrderHash, OrderV4 } from '../../dist/cjs';
import { expect } from 'chai';

describe('Rain Raindex Bindings Package Bindgen Tests', async function () {
	it('should get correct order hash', async () => {
		const order: OrderV4 = {
			owner: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
			evaluable: {
				interpreter: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
				store: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
				bytecode: '0x0102'
			},
			validInputs: [
				{
					token: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					vaultId: '0x0000000000000000000000000000000000000000000000000000000000000000'
				}
			],
			validOutputs: [
				{
					token: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					vaultId: '0x0000000000000000000000000000000000000000000000000000000000001234'
				}
			],
			nonce: '0x0000000000000000000000000000000000000000000000000000000000000002'
		};
		const result = getOrderHash(order);
		if (!result.value) expect.fail('expected to resolve, but did not');
		const expected = '0xce27f4b4eb30405f5c807125e3e7e024acadcce3d77794b5aa644311c17cf272';
		assert.equal(result.value, expected);
	});

	it('should error bad address length', async () => {
		try {
			const order: OrderV4 = {
				owner: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5', // bad length
				evaluable: {
					interpreter: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					store: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					bytecode: '0x0102'
				},
				validInputs: [
					{
						token: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
						vaultId: '0x0000000000000000000000000000000000000000000000000000000000000000'
					}
				],
				validOutputs: [
					{
						token: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
						vaultId: '0x0000000000000000000000000000000000000000000000000000000000001234'
					}
				],
				nonce: '0x0000000000000000000000000000000000000000000000000000000000000002'
			};
			const result = getOrderHash(order);
			if (!result.error) expect.fail('expected to error, but did not');
			assert.fail('expected to error, but resolved');
		} catch (error) {
			assert.ok(error instanceof Error);
			assert.ok(
				error.message.includes('invalid string length'),
				`unexpected error message: ${error.message}`
			);
		}
	});

	it('should error bad vault id', async () => {
		try {
			const order: OrderV4 = {
				owner: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
				evaluable: {
					interpreter: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					store: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					bytecode: '0x0102'
				},
				validInputs: [
					{
						token: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
						vaultId: 'qwe' // bad vault id
					}
				],
				validOutputs: [
					{
						token: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
						vaultId: '0x0000000000000000000000000000000000000000000000000000000000001234'
					}
				],
				nonce: '0x0000000000000000000000000000000000000000000000000000000000000002'
			};
			const result = getOrderHash(order);
			if (!result.error) expect.fail('expected to error, but did not');
			assert.fail('expected to error, but resolved');
		} catch (error) {
			assert.ok(error instanceof Error);
			assert.ok(
				error.message.includes('odd number of digits'),
				`unexpected error message: ${error.message}`
			);
		}
	});

	it('should error bad nonce', async () => {
		try {
			const order: OrderV4 = {
				owner: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
				evaluable: {
					interpreter: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					store: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					bytecode: '0x0102'
				},
				validInputs: [
					{
						token: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
						vaultId: '0x0000000000000000000000000000000000000000000000000000000000000000'
					}
				],
				validOutputs: [
					{
						token: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
						vaultId: '0x0000000000000000000000000000000000000000000000000000000000001234'
					}
				],
				nonce: '0x0000000000000000000000000000000000000000000000000000000000000efg' // bad nonce
			};
			const result = getOrderHash(order);
			if (!result.error) expect.fail('expected to error, but did not');
			assert.fail('expected to error, but resolved');
		} catch (error) {
			assert.ok(error instanceof Error);
			assert.ok(
				error.message.includes("invalid character 'g' at position 63"),
				`unexpected error message: ${error.message}`
			);
		}
	});
});
