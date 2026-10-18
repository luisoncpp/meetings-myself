<script lang="ts">
  interface Props {
    kind: 'done' | 'skipped' | 'missed';
  }

  let { kind }: Props = $props();
</script>

{#if kind === 'done'}
  <svg
    aria-hidden="true"
    width="14"
    height="14"
    viewBox="0 0 14 14"
    fill="none"
    stroke="currentColor"
    stroke-width="1.25"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <path d="M3.25 7.25 5.75 9.75 10.75 4.25" />
  </svg>
{:else if kind === 'skipped'}
  <svg
    aria-hidden="true"
    width="14"
    height="14"
    viewBox="0 0 14 14"
    fill="none"
    stroke="currentColor"
    stroke-width="1.25"
    stroke-linecap="round"
  >
    <path d="M3.5 7h7" />
  </svg>
{:else}
  <svg
    aria-hidden="true"
    width="14"
    height="14"
    viewBox="0 0 14 14"
    fill="none"
    stroke="currentColor"
    stroke-width="1.25"
  >
    <circle cx="7" cy="7" r="3" />
  </svg>
{/if}
