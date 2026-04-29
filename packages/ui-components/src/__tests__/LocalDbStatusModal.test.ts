import { describe, it, expect, afterEach, vi } from 'vitest';
import { render, screen, cleanup } from '@testing-library/svelte';
import LocalDbStatusModal from '../lib/components/LocalDbStatusModal.svelte';
import type { NetworkSyncStatus, RaindexSyncStatus } from '@rainlanguage/raindex';

vi.mock('$lib/utils/getNetworkName', () => ({
	getNetworkName: (chainId: number) => {
		const names: Record<number, string> = {
			137: 'Polygon',
			42161: 'Arbitrum',
			8453: 'Base'
		};
		return names[chainId] ?? null;
	}
}));

describe('LocalDbStatusModal', () => {
	afterEach(() => {
		cleanup();
	});

	it('renders modal with header when open', () => {
		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses: new Map(),
				raindexStatuses: new Map()
			}
		});

		expect(screen.getByText('Database Sync Status')).toBeInTheDocument();
	});

	it('shows empty state message when no networks are syncing', () => {
		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses: new Map(),
				raindexStatuses: new Map()
			}
		});

		expect(screen.getByText('No networks are being synced.')).toBeInTheDocument();
	});

	it('renders network groups with correct network names', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'active', schedulerState: 'leader' }],
			[42161, { chainId: 42161, status: 'syncing', schedulerState: 'leader' }]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				raindexStatuses: new Map()
			}
		});

		expect(screen.getByText('Polygon')).toBeInTheDocument();
		expect(screen.getByText('Arbitrum')).toBeInTheDocument();
	});

	it('falls back to chain ID when network name is unknown', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[99999, { chainId: 99999, status: 'active', schedulerState: 'leader' }]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				raindexStatuses: new Map()
			}
		});

		expect(screen.getByText('Chain 99999')).toBeInTheDocument();
	});

	it('shows Observing badge when schedulerState is notLeader', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'active', schedulerState: 'notLeader' }]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				raindexStatuses: new Map()
			}
		});

		expect(screen.getByText('Observing')).toBeInTheDocument();
	});

	it('does not show Observing badge when schedulerState is leader', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'active', schedulerState: 'leader' }]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				raindexStatuses: new Map()
			}
		});

		expect(screen.queryByText('Observing')).not.toBeInTheDocument();
	});

	it('displays raindex addresses under their network', () => {
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

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				raindexStatuses
			}
		});

		expect(screen.getByText('0x1234567890123456789012345678901234567890')).toBeInTheDocument();
	});

	it('shows phase message when raindex is syncing', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'syncing', schedulerState: 'leader' }]
		]);
		const raindexStatuses = new Map<string, RaindexSyncStatus>([
			[
				'137:0x1234567890123456789012345678901234567890',
				{
					raindexId: {
						chainId: 137,
						raindexAddress: '0x1234567890123456789012345678901234567890'
					},
					status: 'syncing',
					schedulerState: 'leader',
					phaseMessage: 'Fetching latest block'
				}
			]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				raindexStatuses
			}
		});

		expect(screen.getByText('Fetching latest block')).toBeInTheDocument();
	});

	it('does not show phase message when raindex status is not syncing', () => {
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
					schedulerState: 'leader',
					phaseMessage: 'This should not appear'
				}
			]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				raindexStatuses
			}
		});

		expect(screen.queryByText('This should not appear')).not.toBeInTheDocument();
	});

	it('shows error message when raindex has failure', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'failure', schedulerState: 'leader' }]
		]);
		const raindexStatuses = new Map<string, RaindexSyncStatus>([
			[
				'137:0x1234567890123456789012345678901234567890',
				{
					raindexId: {
						chainId: 137,
						raindexAddress: '0x1234567890123456789012345678901234567890'
					},
					status: 'failure',
					schedulerState: 'leader',
					error: 'RPC connection failed'
				}
			]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				raindexStatuses
			}
		});

		expect(screen.getByText('RPC connection failed')).toBeInTheDocument();
	});

	it('shows network-level error when network has failure', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[
				137,
				{
					chainId: 137,
					status: 'failure',
					schedulerState: 'leader',
					error: 'Network initialization failed'
				}
			]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				raindexStatuses: new Map()
			}
		});

		expect(screen.getByText('Network initialization failed')).toBeInTheDocument();
	});

	it('groups raindexes correctly by chain ID', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'active', schedulerState: 'leader' }],
			[42161, { chainId: 42161, status: 'active', schedulerState: 'leader' }]
		]);
		const raindexStatuses = new Map<string, RaindexSyncStatus>([
			[
				'137:0x1111111111111111111111111111111111111111',
				{
					raindexId: {
						chainId: 137,
						raindexAddress: '0x1111111111111111111111111111111111111111'
					},
					status: 'active',
					schedulerState: 'leader'
				}
			],
			[
				'42161:0x2222222222222222222222222222222222222222',
				{
					raindexId: {
						chainId: 42161,
						raindexAddress: '0x2222222222222222222222222222222222222222'
					},
					status: 'syncing',
					schedulerState: 'leader'
				}
			]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				raindexStatuses
			}
		});

		const polygonGroup = screen.getByTestId('network-group-137');
		const arbitrumGroup = screen.getByTestId('network-group-42161');

		expect(polygonGroup).toBeInTheDocument();
		expect(arbitrumGroup).toBeInTheDocument();
		expect(screen.getByText('0x1111111111111111111111111111111111111111')).toBeInTheDocument();
		expect(screen.getByText('0x2222222222222222222222222222222222222222')).toBeInTheDocument();
	});

	it('handles multiple raindexes on the same network', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'active', schedulerState: 'leader' }]
		]);
		const raindexStatuses = new Map<string, RaindexSyncStatus>([
			[
				'137:0x1111111111111111111111111111111111111111',
				{
					raindexId: {
						chainId: 137,
						raindexAddress: '0x1111111111111111111111111111111111111111'
					},
					status: 'active',
					schedulerState: 'leader'
				}
			],
			[
				'137:0x2222222222222222222222222222222222222222',
				{
					raindexId: {
						chainId: 137,
						raindexAddress: '0x2222222222222222222222222222222222222222'
					},
					status: 'syncing',
					schedulerState: 'leader',
					phaseMessage: 'Running bootstrap'
				}
			]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				raindexStatuses
			}
		});

		expect(screen.getByText('0x1111111111111111111111111111111111111111')).toBeInTheDocument();
		expect(screen.getByText('0x2222222222222222222222222222222222222222')).toBeInTheDocument();
		expect(screen.getByText('Running bootstrap')).toBeInTheDocument();
	});

	it('does not show raindex error when status is not failure', () => {
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
					schedulerState: 'leader',
					error: 'This error should not appear'
				}
			]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				raindexStatuses
			}
		});

		expect(screen.queryByText('This error should not appear')).not.toBeInTheDocument();
	});

	it('does not show phase message when schedulerState is notLeader even if status is syncing', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'syncing', schedulerState: 'notLeader' }]
		]);
		const raindexStatuses = new Map<string, RaindexSyncStatus>([
			[
				'137:0x1234567890123456789012345678901234567890',
				{
					raindexId: {
						chainId: 137,
						raindexAddress: '0x1234567890123456789012345678901234567890'
					},
					status: 'syncing',
					schedulerState: 'notLeader',
					phaseMessage: 'Fetching latest block'
				}
			]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				raindexStatuses
			}
		});

		expect(screen.getByText('Observing')).toBeInTheDocument();
		expect(screen.queryByText('Fetching latest block')).not.toBeInTheDocument();
	});
});
