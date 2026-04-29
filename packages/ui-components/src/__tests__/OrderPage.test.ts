import { render, screen, waitFor } from '@testing-library/svelte';
import OrderPage from '../lib/components/deployment/OrderPage.svelte';
import type { NameAndDescriptionCfg } from '@rainlanguage/raindex';
import { vi, describe, it, expect, beforeEach } from 'vitest';

// Mock fetch
const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

vi.mock('../lib/components/deployment/DeploymentsSection.svelte', async () => {
	const MockDeploymentsSection = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { default: MockDeploymentsSection };
});

describe('OrderPage', () => {
	beforeEach(() => {
		mockFetch.mockReset();
	});

	it('renders order details when provided directly', async () => {
		render(OrderPage, {
			props: {
				orderName: 'TestOrder',
				orderDetail: {
					name: 'Test Order',
					description: 'Test Description',
					short_description: 'Test Short Description'
				}
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Test Order')).toBeInTheDocument();
			expect(screen.getByText('Test Description')).toBeInTheDocument();
		});
	});

	it('renders markdown when description is a markdown url', async () => {
		mockFetch.mockResolvedValueOnce({
			ok: true,
			text: () => Promise.resolve('mock markdown content')
		});

		render(OrderPage, {
			props: {
				orderName: 'TestOrder',
				orderDetail: {
					name: 'Test Order',
					description: 'https://example.com/description.md',
					short_description: 'Test Short Description'
				}
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Test Order')).toBeInTheDocument();
			expect(screen.getByTestId('markdown-content')).toBeInTheDocument();
			expect(mockFetch).toHaveBeenCalledWith('https://example.com/description.md');
		});
	});

	it('handles markdown fetch failure', async () => {
		mockFetch.mockRejectedValueOnce(new Error('Failed to fetch'));

		render(OrderPage, {
			props: {
				orderName: 'TestOrder',
				orderDetail: {
					name: 'Test Order',
					description: 'https://example.com/description.md',
					short_description: 'Test Short Description'
				}
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Failed to fetch markdown')).toBeInTheDocument();
		});
	});

	it('falls back to plain text when markdown fetch is not ok', async () => {
		mockFetch.mockResolvedValueOnce({
			ok: false,
			statusText: 'Not Found'
		});

		render(OrderPage, {
			props: {
				orderName: 'TestOrder',
				orderDetail: {
					name: 'Test Order',
					description: 'https://example.com/description.md',
					short_description: 'Test Short Description'
				}
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Test Order')).toBeInTheDocument();
			expect(screen.getByTestId('plain-description')).toHaveTextContent(
				'https://example.com/description.md'
			);
		});
	});

	it('shows fallback when order detail is missing', () => {
		render(OrderPage, {
			// Casting to satisfy the required prop in tests
			props: {
				orderName: 'TestOrder',
				orderDetail: undefined as unknown as NameAndDescriptionCfg
			}
		});

		expect(screen.getByText('Failed to load order details.')).toBeInTheDocument();
	});
});
