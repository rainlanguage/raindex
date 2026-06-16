import { RainlangManager } from '../providers/rainlang/RainlangManager';
import { DotrainRainlang } from '@rainlanguage/orderbook';

export async function loadRainlangUrl(
	url: string,
	rainlangManager: RainlangManager
): Promise<void> {
	if (!url) {
		throw new Error('No URL provided');
	}

	if (!rainlangManager) {
		throw new Error('Rainlang manager is required');
	}

	try {
		const validationResult = await DotrainRainlang.validate(url);
		if (validationResult.error) {
			throw new Error(validationResult.error.readableMsg);
		}
		rainlangManager.setRainlang(url);
		window.location.reload();
	} catch (e) {
		const errorMessage = e instanceof Error ? e.message : 'Failed to update rainlang URL';
		throw new Error(errorMessage);
	}
}
