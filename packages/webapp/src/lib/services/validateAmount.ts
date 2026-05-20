import fc from 'fast-check';
import { test } from '@fast-check/vitest';
import { Float } from '@rainlanguage/raindex';

/**
 * Validates an amount against a balance and returns validation results
 *
 * @param amount The amount being entered
 * @param balance The available balance
 * @returns Validation results object
 */

export function validateAmount(
	amount: Float,
	balance: Float
): {
	isValid: boolean;
	isZero: boolean;
	exceedsBalance: boolean;
	errorMessage: string | null;
} {
	const zero = Float.parse('0').value as Float;

	const isZero = amount.lte(zero).value;
	const exceedsBalance = amount.gt(balance).value;
	const isValid = !isZero && !exceedsBalance;

	let errorMessage = null;
	if (isZero) {
		errorMessage = 'Amount must be greater than zero.';
	} else if (exceedsBalance) {
		errorMessage = 'Amount cannot exceed available balance.';
	}

	return {
		isValid,
		isZero: !!isZero,
		exceedsBalance: !!exceedsBalance,
		errorMessage
	};
}

if (import.meta.vitest) {
	const { expect, it } = import.meta.vitest;

	test.prop([
		fc.bigInt().map((n) => Float.fromFixedDecimal(n, 18)),
		fc.bigInt().map((n) => Float.fromFixedDecimal(n, 18))
	])('validates amounts against balances correctly', (amountRes, balanceRes) => {
		if (amountRes.error || balanceRes.error) return;
		const amount = amountRes.value;
		const balance = balanceRes.value;

		const result = validateAmount(amount, balance);
		const zero = Float.parse('0').value as Float;

		expect(result.isZero).toBe(amount.lte(zero).value);
		expect(result.exceedsBalance).toBe(amount.gt(balance).value);
		expect(result.isValid).toBe(amount.gt(zero).value && amount.lte(balance).value);

		if (amount.lte(zero).value) {
			expect(result.isValid).toBe(false);
			expect(result.errorMessage).toContain('greater than zero');
		} else if (amount.gt(balance).value) {
			expect(result.isValid).toBe(false);
			expect(result.errorMessage).toContain('exceed available balance');
		} else {
			expect(result.isValid).toBe(true);
			expect(result.errorMessage).toBeNull();
		}
	});

	it('handles edge cases correctly', () => {
		// Test with zero amount
		expect(
			validateAmount(Float.parse('0').value as Float, Float.parse('100').value as Float)
		).toEqual({
			isValid: false,
			isZero: true,
			exceedsBalance: false,
			errorMessage: 'Amount must be greater than zero.'
		});

		// Test with negative amount
		expect(
			validateAmount(Float.parse('-1').value as Float, Float.parse('100').value as Float)
		).toEqual({
			isValid: false,
			isZero: true,
			exceedsBalance: false,
			errorMessage: 'Amount must be greater than zero.'
		});

		// Test with amount equal to balance
		expect(
			validateAmount(Float.parse('100').value as Float, Float.parse('100').value as Float)
		).toEqual({
			isValid: true,
			isZero: false,
			exceedsBalance: false,
			errorMessage: null
		});

		// Test with very large numbers
		const maxBigInt = BigInt(Number.MAX_SAFE_INTEGER);
		expect(
			validateAmount(
				Float.fromFixedDecimal(maxBigInt, 18).value as Float,
				Float.fromFixedDecimal(maxBigInt, 18).value as Float
			)
		).toEqual({
			isValid: true,
			isZero: false,
			exceedsBalance: false,
			errorMessage: null
		});
	});
}
