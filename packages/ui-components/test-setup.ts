import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

window.scrollTo = vi.fn();

vi.mock('codemirror-rainlang', () => ({
	RainlangLR: vi.fn()
}));

vi.mock('$app/stores', async () => {
	const { readable, writable } = await import('svelte/store');
	/**
	 * @type {import('$app/stores').getStores}
	 */
	const getStores = () => ({
		navigating: readable(null),
		page: readable({
			url: new URL('http://localhost'),
			params: {},
			searchParams: new URLSearchParams()
		}),
		session: writable(null),
		updated: readable(false)
	});
	/** @type {typeof import('$app/stores').page} */
	const page = {
		subscribe(fn) {
			return getStores().page.subscribe(fn);
		}
	};
	/** @type {typeof import('$app/stores').navigating} */
	const navigating = {
		subscribe(fn) {
			return getStores().navigating.subscribe(fn);
		}
	};
	/** @type {typeof import('$app/stores').session} */
	const session = {
		subscribe(fn) {
			return getStores().session.subscribe(fn);
		}
	};
	/** @type {typeof import('$app/stores').updated} */
	const updated = {
		subscribe(fn) {
			return getStores().updated.subscribe(fn);
		}
	};
	return {
		getStores,
		navigating,
		page,
		session,
		updated
	};
});

vi.mock('@rainlanguage/orderbook', () => {
	const RaindexOrderBuilder = vi.fn();
	// @ts-expect-error - static method
	RaindexOrderBuilder.getOrderDetails = vi.fn();
	// @ts-expect-error - static method
	RaindexOrderBuilder.getDeploymentDetails = vi.fn();
	RaindexOrderBuilder.prototype.newWithDeployment = vi.fn();
	RaindexOrderBuilder.prototype.getOrderDetails = vi.fn();
	RaindexOrderBuilder.prototype.setVaultId = vi.fn();
	RaindexOrderBuilder.prototype.getTokenInfo = vi.fn();
	RaindexOrderBuilder.prototype.getCurrentDeployment = vi.fn();
	RaindexOrderBuilder.prototype.getVaultIds = vi.fn();
	RaindexOrderBuilder.prototype.setDeposit = vi.fn();
	RaindexOrderBuilder.prototype.getDeposits = vi.fn();
	RaindexOrderBuilder.prototype.setFieldValue = vi.fn();
	RaindexOrderBuilder.prototype.getFieldValue = vi.fn();
	RaindexOrderBuilder.prototype.getSelectTokens = vi.fn();
	RaindexOrderBuilder.prototype.getAllTokenInfos = vi.fn();
	RaindexOrderBuilder.prototype.getAllFieldDefinitions = vi.fn();
	RaindexOrderBuilder.prototype.isSelectTokenSet = vi.fn();
	RaindexOrderBuilder.prototype.setSelectToken = vi.fn();
	RaindexOrderBuilder.prototype.unsetSelectToken = vi.fn();
	RaindexOrderBuilder.prototype.hasAnyDeposit = vi.fn();
	RaindexOrderBuilder.prototype.hasAnyVaultId = vi.fn();
	RaindexOrderBuilder.prototype.areAllTokensSelected = vi.fn();
	RaindexOrderBuilder.prototype.getDeploymentTransactionArgs = vi.fn();
	RaindexOrderBuilder.prototype.generateApprovalCalldatas = vi.fn();
	RaindexOrderBuilder.prototype.serializeState = vi.fn();
	RaindexOrderBuilder.prototype.getAllBuilderConfig = vi.fn();
	RaindexOrderBuilder.prototype.getCurrentDeploymentDetails = vi.fn();
	return {
		RaindexOrderBuilder
	};
});
