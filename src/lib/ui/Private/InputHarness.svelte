<script lang="ts">
  import Field from './Field.svelte';
  import Input from './Input.svelte';

  let value = $state('');
</script>

<Field label="Task name" forId="task-input">
  <Input id="task-input" bind:value />
</Field>
