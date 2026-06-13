<script lang="ts">
	import { afterUpdate } from 'svelte';
	import { props } from './MockComponent';

	// `testid` lets a test that mounts more than one MockComponent give each
	// instance a distinct `data-testid`, so they can be queried individually.
	// It defaults to "mock-component" for the single-instance case.
	export let testid: string = 'mock-component';

	// The received props are mirrored into a shared store so tests can assert on
	// values (functions, objects) that cannot round-trip through DOM attributes.
	// This runs in `afterUpdate` rather than a reactive `$:` block so it never
	// re-enters the component's update cycle, which caused the invalid array
	// length crash when multiple instances were mounted.
	afterUpdate(() => {
		props.set($$restProps);
	});
</script>

<div data-testid={testid} {...$$restProps}>
	<slot />
</div>
