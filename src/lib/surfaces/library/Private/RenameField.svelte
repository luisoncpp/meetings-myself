<script lang="ts">
  import { Field, Input } from '../../../ui';

  interface Props {
    title: string;
    label: string;
    onrename: (title: string) => void;
  }

  let { title, label, onrename }: Props = $props();

  let draft = $derived(title);

  function commit(): void {
    const trimmed = draft.trim();
    if (trimmed === '' || trimmed === title) return;
    onrename(trimmed);
  }
</script>

<Field {label}>
  <Input bind:value={draft} aria-label={label} onchange={/* rename */ commit} />
</Field>
