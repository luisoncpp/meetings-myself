<script lang="ts">
  import SurfaceLayout from './SurfaceLayout.svelte';

  interface Props {
    class?: string;
    'aria-labelledby'?: string;
    content?: string;
  }

  let { class: className, 'aria-labelledby': ariaLabelledby, content = 'Surface content' }: Props =
    $props();
</script>

<SurfaceLayout class={className} aria-labelledby={ariaLabelledby}>
  {content}
</SurfaceLayout>
