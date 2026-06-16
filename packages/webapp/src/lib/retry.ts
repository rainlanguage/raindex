export const DEFAULT_MAX_RETRIES = 3;

export async function retry<T>(fn: () => Promise<T>, retries = DEFAULT_MAX_RETRIES): Promise<T> {
	for (let i = 0; i < retries; i++) {
		try {
			return await fn();
		} catch (e) {
			if (i === retries - 1) throw e;
			await new Promise((r) => setTimeout(r, 1000 * (i + 1)));
		}
	}
	throw new Error('unreachable');
}
