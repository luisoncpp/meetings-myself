<script lang="ts">
  import { Field, Input } from '../../../ui';

  interface Props {
    onsubmit: (title: string) => void;
  }

  let { onsubmit }: Props = $props();

  let draft = $state('');

  function submit(): void {
    const title = draft.trim();
    if (title === '') return;
    onsubmit(title);
    draft = '';
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key !== 'Enter') return;
    event.preventDefault();
    submit();
  }
</script>

<div class="quick-add">
  <Field label="Add a task" forId="quick-add">
    <Input
      id="quick-add"
      type="text"
      bind:value={draft}
      onkeydown={handleKeydown}
      autocomplete="off"
    />
  </Field>
</div>

<style>
  .quick-add {
    margin-bottom: var(--space-3);
  }
</style>
