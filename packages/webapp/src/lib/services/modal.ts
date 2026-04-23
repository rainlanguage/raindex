import DepositModal from '$lib/components/DepositModal.svelte';
import WithdrawModal from '$lib/components/WithdrawModal.svelte';
import TransactionConfirmationModal from '$lib/components/TransactionConfirmationModal.svelte';
import TakeOrderModal from '$lib/components/TakeOrderModal.svelte';
import {
	DisclaimerModal,
	type TransactionConfirmationProps,
	type DisclaimerModalProps,
	type VaultActionModalProps
} from '@rainlanguage/ui-components';
import WithdrawAllModal from '../components/WithdrawAllModal.svelte';
import type { WithdrawAllModalProps } from './handleVaultsWithdrawAll';
import type { RaindexOrder, RaindexOrderQuote, TakeOrdersMode } from '@rainlanguage/orderbook';

export const handleDepositModal = (props: VaultActionModalProps) => {
	new DepositModal({ target: document.body, props });
};

export const handleWithdrawModal = (props: VaultActionModalProps) => {
	new WithdrawModal({ target: document.body, props });
};

export const handleWithdrawAllModal = (props: WithdrawAllModalProps) => {
	new WithdrawAllModal({ target: document.body, props });
};

export type TransactionConfirmationModalResult = {
	success: boolean;
	hash?: string;
};

export const handleTransactionConfirmationModal = (
	props: TransactionConfirmationProps
): Promise<TransactionConfirmationModalResult> => {
	return new Promise((resolve) => {
		const originalOnConfirm = props.args.onConfirm;
		let modalResolved = false;

		// Wrap the onConfirm to resolve our promise
		props.args.onConfirm = (hash) => {
			originalOnConfirm(hash);
			if (!modalResolved) {
				modalResolved = true;
				resolve({ success: true, hash });
			}
		};

		// Create modal with modified props
		const modal = new TransactionConfirmationModal({
			target: document.body,
			props: {
				...props,
				closeOnConfirm: true
			}
		});

		// Check periodically if modal was dismissed
		const checkDismissal = setInterval(() => {
			if (!modal.$$.ctx || modal.$$.destroyed) {
				if (!modalResolved) {
					modalResolved = true;
					resolve({ success: false });
				}
				clearInterval(checkDismissal);
			}
		}, 500);

		// Clean up after 30 seconds maximum
		setTimeout(() => {
			if (!modalResolved) {
				modalResolved = true;
				resolve({ success: false });
			}
			clearInterval(checkDismissal);
			if (modal && !modal.$$.destroyed) {
				modal.$destroy();
			}
		}, 30000);
	});
};

export const handleDisclaimerModal = (props: DisclaimerModalProps) => {
	new DisclaimerModal({ target: document.body, props });
};

export interface TakeOrderSubmitParams {
	quote: RaindexOrderQuote;
	mode: TakeOrdersMode;
	amount: string;
	priceCap: string;
}

export interface TakeOrderModalProps {
	open: boolean;
	order: RaindexOrder;
	onSubmit: (params: TakeOrderSubmitParams) => Promise<void>;
}

export type HandleTakeOrderModal = (props: TakeOrderModalProps) => void;

export const handleTakeOrderModal: HandleTakeOrderModal = (props: TakeOrderModalProps) => {
	new TakeOrderModal({ target: document.body, props });
};
