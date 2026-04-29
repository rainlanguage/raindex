import type {
	RaindexTrade,
	RaindexTransaction,
	RaindexVaultBalanceChange,
	RaindexVaultToken
} from '@rainlanguage/raindex';
import type { UTCTimestamp } from 'lightweight-charts';
import { timestampSecondsToUTCTimestamp } from '../services/time';
import { sortBy } from 'lodash';

export type HistoricalOrderChartData = { value: number; time: UTCTimestamp; color?: string }[];

export function prepareHistoricalOrderChartData(
	takeOrderEntities: RaindexTrade[],
	colorTheme: string
) {
	const transformedData = takeOrderEntities.map((d) => ({
		value: Math.abs(
			Number(d.inputVaultBalanceChange.formattedAmount) /
				Number(d.outputVaultBalanceChange.formattedAmount)
		),
		time: timestampSecondsToUTCTimestamp(BigInt(d.timestamp)),
		color: colorTheme == 'dark' ? '#5178FF' : '#4E4AF6',
		outputAmount: Number(d.outputVaultBalanceChange.amount)
	}));

	// if we have multiple object in the array with the same timestamp, we need to merge them
	// we do this by taking the weighted average of the ioratio values for objects that share the same timestamp.
	const uniqueTimestamps = Array.from(new Set(transformedData.map((d) => d.time)));
	const finalData: HistoricalOrderChartData = [];
	uniqueTimestamps.forEach((timestamp) => {
		const objectsWithSameTimestamp = transformedData.filter((d) => d.time === timestamp);
		if (objectsWithSameTimestamp.length > 1) {
			// calculate a weighted average of the ioratio values using the amount of the output token as the weight
			const ioratioSum = objectsWithSameTimestamp.reduce(
				(acc, d) => acc + d.value * d.outputAmount,
				0
			);
			const outputAmountSum = objectsWithSameTimestamp.reduce((acc, d) => acc + d.outputAmount, 0);
			const ioratioAverage = ioratioSum / outputAmountSum;
			finalData.push({
				value: ioratioAverage,
				time: timestamp,
				color: objectsWithSameTimestamp[0].color
			});
		} else {
			finalData.push(objectsWithSameTimestamp[0]);
		}
	});

	return sortBy(finalData, (d) => d.time);
}

if (import.meta.vitest) {
	const { it, expect } = import.meta.vitest;

	it('transforms and sorts data as expected', () => {
		const takeOrderEntities: RaindexTrade[] = [
			{
				id: '1',
				timestamp: BigInt(1632000000),
				transaction: {
					id: 'transaction_id',
					from: '0xsender_address',
					timestamp: BigInt(1632000000),
					blockNumber: BigInt(0)
				} as unknown as RaindexTransaction,
				outputVaultBalanceChange: {
					amount: BigInt(100),
					formattedAmount: '100',
					vaultId: BigInt(1),
					__typename: 'Withdraw',
					token: {
						id: 'output_token',
						address: '0xoutput_token',
						name: 'output_token',
						symbol: 'output_token',
						decimals: BigInt(1)
					} as unknown as RaindexVaultToken,
					newBalance: BigInt(0),
					formattedNewBalance: '0',
					oldBalance: BigInt(0),
					formattedOldBalance: '0',
					timestamp: BigInt(0),
					transaction: {
						id: 'transaction_id',
						from: 'sender_address',
						timestamp: BigInt(1632000000),
						blockNumber: BigInt(0)
					} as unknown as RaindexTransaction,
					raindex: '0x1'
				} as unknown as RaindexVaultBalanceChange,
				orderHash: 'orderHash',
				inputVaultBalanceChange: {
					vaultId: BigInt(1),
					token: {
						id: 'output_token',
						address: '0xoutput_token',
						name: 'output_token',
						symbol: 'output_token',
						decimals: BigInt(1)
					} as unknown as RaindexVaultToken,
					amount: BigInt(50),
					formattedAmount: '50',
					__typename: 'Withdraw',
					newBalance: BigInt(0),
					formattedNewBalance: '0',
					oldBalance: BigInt(0),
					formattedOldBalance: '0',
					timestamp: BigInt(0),
					transaction: {
						id: 'transaction_id',
						from: '0xsender_address',
						timestamp: BigInt(1632000000),
						blockNumber: BigInt(0)
					} as unknown as RaindexTransaction,
					raindex: '0x1'
				} as unknown as RaindexVaultBalanceChange,
				raindex: '0x00'
			} as unknown as RaindexTrade,
			{
				id: '2',
				timestamp: BigInt(1631000000),
				transaction: {
					id: 'transaction_id',
					from: '0xsender_address',
					timestamp: BigInt(1631000000),
					blockNumber: BigInt(0)
				} as unknown as RaindexTransaction,
				outputVaultBalanceChange: {
					amount: BigInt(100),
					formattedAmount: '100',
					vaultId: BigInt(1),
					token: {
						id: 'output_token',
						address: '0xoutput_token',
						name: 'output_token',
						symbol: 'output_token',
						decimals: BigInt(1)
					} as unknown as RaindexVaultToken,
					newBalance: BigInt(0),
					formattedNewBalance: '0',
					oldBalance: BigInt(0),
					formattedOldBalance: '0',
					__typename: 'Withdraw',
					timestamp: BigInt(0),
					transaction: {
						id: 'transaction_id',
						from: '0xsender_address',
						timestamp: BigInt(1632000000),
						blockNumber: BigInt(0)
					} as unknown as RaindexTransaction,
					raindex: '0x1'
				} as unknown as RaindexVaultBalanceChange,
				orderHash: 'orderHash',
				inputVaultBalanceChange: {
					vaultId: BigInt(1),
					token: {
						id: 'output_token',
						address: '0xoutput_token',
						name: 'output_token',
						symbol: 'output_token',
						decimals: BigInt(1)
					} as unknown as RaindexVaultToken,
					amount: BigInt(50),
					formattedAmount: '50',
					__typename: 'Withdraw',
					newBalance: BigInt(0),
					formattedNewBalance: '0',
					oldBalance: BigInt(0),
					formattedOldBalance: '0',
					timestamp: BigInt(0),
					transaction: {
						id: 'transaction_id',
						from: '0xsender_address',
						timestamp: BigInt(1632000000),
						blockNumber: BigInt(0)
					} as unknown as RaindexTransaction,
					raindex: '0x1'
				} as unknown as RaindexVaultBalanceChange,
				raindex: '0x00'
			} as unknown as RaindexTrade,
			{
				id: '3',
				timestamp: BigInt(1630000000),
				transaction: {
					id: 'transaction_id',
					from: '0xsender_address',
					timestamp: BigInt(1630000000),
					blockNumber: BigInt(0)
				} as unknown as RaindexTransaction,
				outputVaultBalanceChange: {
					amount: BigInt(100),
					formattedAmount: '100',
					vaultId: BigInt(1),
					token: {
						id: 'output_token',
						address: '0xoutput_token',
						name: 'output_token',
						symbol: 'output_token',
						decimals: BigInt(1)
					} as unknown as RaindexVaultToken,
					newBalance: BigInt(0),
					formattedNewBalance: '0',
					oldBalance: BigInt(0),
					formattedOldBalance: '0',
					__typename: 'Withdraw',
					timestamp: BigInt(0),
					transaction: {
						id: 'transaction_id',
						from: 'sender_address',
						timestamp: BigInt(1632000000),
						blockNumber: BigInt(0)
					} as unknown as RaindexTransaction,
					raindex: '0x1'
				} as unknown as RaindexVaultBalanceChange,
				orderHash: 'orderHash',
				inputVaultBalanceChange: {
					vaultId: BigInt(1),
					token: {
						id: 'output_token',
						address: '0xoutput_token',
						name: 'output_token',
						symbol: 'output_token',
						decimals: BigInt(1)
					} as unknown as RaindexVaultToken,
					newBalance: BigInt(0),
					formattedNewBalance: '0',
					oldBalance: BigInt(0),
					formattedOldBalance: '0',
					amount: BigInt(50),
					formattedAmount: '50',
					__typename: 'Withdraw',
					timestamp: BigInt(0),
					transaction: {
						id: 'transaction_id',
						from: 'sender_address',
						timestamp: BigInt(1632000000),
						blockNumber: BigInt(0)
					} as unknown as RaindexTransaction,
					raindex: '0x1'
				} as unknown as RaindexVaultBalanceChange,
				raindex: '0x00'
			} as unknown as RaindexTrade
		];

		const result = prepareHistoricalOrderChartData(takeOrderEntities, 'dark');

		expect(result.length).toEqual(3);
		expect(result[0].value).toEqual(0.5);
		expect(result[0].time).toEqual(1630000000);
		expect(result[1].value).toEqual(0.5);
		expect(result[1].time).toEqual(1631000000);
		expect(result[2].value).toEqual(0.5);
		expect(result[2].time).toEqual(1632000000);

		// check the color
		expect(result[0].color).toEqual('#5178FF');
		expect(result[1].color).toEqual('#5178FF');
		expect(result[2].color).toEqual('#5178FF');
	});

	it('handles the case where multiple trades have the same timestamp', () => {
		const takeOrderEntities: RaindexTrade[] = [
			{
				id: '1',
				timestamp: BigInt(1632000000),
				transaction: {
					id: 'transaction_id',
					from: '0xsender_address',
					timestamp: BigInt(1632000000),
					blockNumber: BigInt(0)
				} as unknown as RaindexTransaction,
				outputVaultBalanceChange: {
					amount: BigInt(100),
					formattedAmount: '100',
					vaultId: BigInt(1),
					token: {
						id: 'output_token',
						address: '0xoutput_token',
						name: 'output_token',
						symbol: 'output_token',
						decimals: BigInt(1)
					} as unknown as RaindexVaultToken,
					newBalance: BigInt(0),
					formattedNewBalance: '0',
					oldBalance: BigInt(0),
					formattedOldBalance: '0',
					timestamp: BigInt(0),
					transaction: {
						id: 'transaction_id',
						from: '0xsender_address',
						timestamp: BigInt(1632000000),
						blockNumber: BigInt(0)
					} as unknown as RaindexTransaction,
					__typename: 'Withdraw',
					raindex: '0x1'
				} as unknown as RaindexVaultBalanceChange,
				orderHash: 'orderHash',
				inputVaultBalanceChange: {
					vaultId: BigInt(1),
					token: {
						id: 'output_token',
						address: '0xoutput_token',
						name: 'output_token',
						symbol: 'output_token',
						decimals: BigInt(1)
					} as unknown as RaindexVaultToken,
					amount: BigInt(50),
					formattedAmount: '50',
					newBalance: BigInt(0),
					formattedNewBalance: '0',
					oldBalance: BigInt(0),
					formattedOldBalance: '0',
					timestamp: BigInt(0),
					transaction: {
						id: 'transaction_id',
						from: '0xsender_address',
						timestamp: BigInt(1632000000),
						blockNumber: BigInt(0)
					} as unknown as RaindexTransaction,
					raindex: '0x1'
				} as unknown as RaindexVaultBalanceChange,
				raindex: '0x00'
			} as unknown as RaindexTrade,
			{
				id: '2',
				timestamp: BigInt(1632000000),
				transaction: {
					id: 'transaction_id',
					from: '0xsender_address',
					timestamp: BigInt(1632000000),
					blockNumber: BigInt(0)
				} as unknown as RaindexTransaction,
				outputVaultBalanceChange: {
					amount: BigInt(200),
					formattedAmount: '200',
					vaultId: BigInt(1),
					token: {
						id: 'output_token',
						address: '0xoutput_token',
						name: 'output_token',
						symbol: 'output_token',
						decimals: BigInt(1)
					} as unknown as RaindexVaultToken,
					newBalance: BigInt(0),
					formattedNewBalance: '0',
					oldBalance: BigInt(0),
					formattedOldBalance: '0',
					timestamp: BigInt(0),
					transaction: {
						id: 'transaction_id',
						from: '0xsender_address',
						timestamp: BigInt(1632000000),
						blockNumber: BigInt(0)
					} as unknown as RaindexTransaction,
					raindex: '0x1'
				} as unknown as RaindexVaultBalanceChange,
				orderHash: 'orderHash',
				inputVaultBalanceChange: {
					vaultId: BigInt(1),
					token: {
						id: 'output_token',
						address: '0xoutput_token',
						name: 'output_token',
						symbol: 'output_token',
						decimals: BigInt(1)
					} as unknown as RaindexVaultToken,
					amount: BigInt(50),
					formattedAmount: '50',
					newBalance: BigInt(0),
					formattedNewBalance: '0',
					oldBalance: BigInt(0),
					formattedOldBalance: '0',
					timestamp: BigInt(0),
					transaction: {
						id: 'transaction_id',
						from: '0xsender_address',
						timestamp: BigInt(1632000000),
						blockNumber: BigInt(0)
					} as unknown as RaindexTransaction,
					raindex: '0x1'
				} as unknown as RaindexVaultBalanceChange,
				raindex: '0x00'
			} as unknown as RaindexTrade,
			{
				id: '3',
				timestamp: BigInt(1632000000),
				transaction: {
					id: 'transaction_id',
					from: '0xsender_address',
					timestamp: BigInt(1632000000),
					blockNumber: BigInt(0)
				} as unknown as RaindexTransaction,
				outputVaultBalanceChange: {
					amount: BigInt(400),
					formattedAmount: '400',
					vaultId: BigInt(1),
					token: {
						id: 'output_token',
						address: '0xoutput_token',
						name: 'output_token',
						symbol: 'output_token',
						decimals: BigInt(1)
					} as unknown as RaindexVaultToken,
					newBalance: BigInt(0),
					formattedNewBalance: '0',
					oldBalance: BigInt(0),
					formattedOldBalance: '0',
					timestamp: BigInt(0),
					transaction: {
						id: 'transaction_id',
						from: '0xsender_address',
						timestamp: BigInt(1632000000),
						blockNumber: BigInt(0)
					} as unknown as RaindexTransaction,
					raindex: '0x1'
				} as unknown as RaindexVaultBalanceChange,
				orderHash: 'orderHash',
				inputVaultBalanceChange: {
					vaultId: BigInt(1),
					token: {
						id: 'output_token_id',
						address: '0xoutput_token',
						name: 'output_token',
						symbol: 'output_token',
						decimals: BigInt(1)
					} as unknown as RaindexVaultToken,
					amount: BigInt(50),
					formattedAmount: '50',
					newBalance: BigInt(0),
					formattedNewBalance: '0',
					oldBalance: BigInt(0),
					formattedOldBalance: '0',
					timestamp: BigInt(0),
					transaction: {
						id: 'transaction_id',
						from: '0xsender_address',
						timestamp: BigInt(1632000000),
						blockNumber: BigInt(0)
					} as unknown as RaindexTransaction,
					raindex: '0x1'
				} as unknown as RaindexVaultBalanceChange,
				raindex: '0x00'
			} as unknown as RaindexTrade
		];

		const result = prepareHistoricalOrderChartData(takeOrderEntities, 'dark');

		// calculate the weighted average of the ioratio values
		const ioratioSum = 0.5 * 100 + 0.25 * 200 + 0.125 * 400;
		const outputAmountSum = 100 + 200 + 400;
		const ioratioAverage = ioratioSum / outputAmountSum;

		expect(result.length).toEqual(1);
		expect(result[0].value).toEqual(ioratioAverage);
	});
}
