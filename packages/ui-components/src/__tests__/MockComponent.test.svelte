<script lang="ts">
	import MockComponent from '../lib/__mocks__/MockComponent.svelte';
</script>

<MockComponent testid="first" source="a">
	<span>first slot</span>
</MockComponent>
<MockComponent testid="second" source="b">
	<span>second slot</span>
</MockComponent>
