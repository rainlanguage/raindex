import { render, screen, cleanup, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, afterEach, vi } from 'vitest';
import LocalDbStatusCard from '../lib/components/LocalDbStatusCard.svelte';
import type { NetworkSyncStatus, RaindexSyncStatus } from '@rainlanguage/raindex';

vi.mock('$lib/utils/getNetworkName', () => ({
	getNetworkName: (chainId: number) => {
		const names: Record<number, string> = {
			1: 'Ethereum',
			137: 'Polygon',
			42161: 'Arbitrum'
		};
		return names[chainId] ?? null;
	}
}));

describe('LocalDbStatusCard', () => {
	afterEach(() => {
		cleanup();
	});

	it('renders the default label and badge', () => {
		render(LocalDbStatusCard);

		expect(screen.getByText('LocalDB')).toBeInTheDocument();
		expect(screen.getByTestId('local-db-status')).toBeInTheDocument();
	});

	it('shows active status when no networks have failures', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[1, { chainId: 1, status: 'active', schedulerState: 'leader' }],
			[137, { chainId: 137, status: 'syncing', schedulerState: 'leader' }]
		]);

		render(LocalDbStatusCard, {
			props: { networkStatuses }
		});

		expect(screen.getByText('LocalDB')).toBeInTheDocument();
		expect(screen.getByText('Active')).toBeInTheDocument();
	});

	it('shows failure status when any network has a failure', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[1, { chainId: 1, status: 'active', schedulerState: 'leader' }],
			[137, { chainId: 137, status: 'failure', schedulerState: 'leader', error: 'RPC error' }]
		]);

		render(LocalDbStatusCard, {
			props: { networkStatuses }
		});

		expect(screen.getByText('LocalDB')).toBeInTheDocument();
		expect(screen.getByText('Failure')).toBeInTheDocument();
	});

	it('shows chevron when networks are present', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[1, { chainId: 1, status: 'active', schedulerState: 'leader' }]
		]);

		render(LocalDbStatusCard, {
			props: { networkStatuses }
		});

		expect(screen.getByTestId('local-db-status-header')).toBeInTheDocument();
	});

	it('shows active status when all networks are syncing but none failing', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'syncing', schedulerState: 'leader' }],
			[42161, { chainId: 42161, status: 'syncing', schedulerState: 'leader' }]
		]);

		render(LocalDbStatusCard, {
			props: { networkStatuses }
		});

		expect(screen.getByText('Active')).toBeInTheDocument();
	});

	it('shows active status when empty maps are provided', () => {
		render(LocalDbStatusCard, {
			props: {
				networkStatuses: new Map(),
				raindexStatuses: new Map()
			}
		});

		expect(screen.getByText('Active')).toBeInTheDocument();
	});

	it('accepts raindexStatuses prop', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'active', schedulerState: 'leader' }]
		]);
		const raindexStatuses = new Map<string, RaindexSyncStatus>([
			[
				'137:0x1234567890123456789012345678901234567890',
				{
					raindexId: {
						chainId: 137,
						raindexAddress: '0x1234567890123456789012345678901234567890'
					},
					status: 'active',
					schedulerState: 'leader'
				}
			]
		]);

		render(LocalDbStatusCard, {
			props: { networkStatuses, raindexStatuses }
		});

		expect(screen.getByText('Active')).toBeInTheDocument();
	});

	it('shows failure when first network is active but second has failure', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[1, { chainId: 1, status: 'active', schedulerState: 'leader' }],
			[137, { chainId: 137, status: 'active', schedulerState: 'leader' }],
			[42161, { chainId: 42161, status: 'failure', schedulerState: 'leader', error: 'Timeout' }]
		]);

		render(LocalDbStatusCard, {
			props: { networkStatuses }
		});

		expect(screen.getByText('Failure')).toBeInTheDocument();
	});

	it('opens modal when header button is clicked', async () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'active', schedulerState: 'leader' }]
		]);

		render(LocalDbStatusCard, {
			props: { networkStatuses }
		});

		const headerButton = screen.getByTestId('local-db-status-header');
		await fireEvent.click(headerButton);

		expect(screen.getByTestId('local-db-status-modal')).toBeInTheDocument();
	});

	it('renders data-testid on card container', () => {
		render(LocalDbStatusCard);

		expect(screen.getByTestId('local-db-status-card')).toBeInTheDocument();
	});
});
