<script lang="ts">
  import OrderableList from './OrderableList.svelte';

  interface Props {
    items: string[];
    onreorder: (order: string[]) => void;
  }

  let { items, onreorder }: Props = $props();
</script>

<OrderableList items={items} getId={(item) => item} {onreorder}>
  {#snippet children(item)}
    {item}
  {/snippet}
</OrderableList>
