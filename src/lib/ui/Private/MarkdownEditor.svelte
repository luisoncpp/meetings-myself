<script lang="ts">
  import { t } from '../../i18n';
  import 'quill-next/dist/quill.snow.css';
  import { onMount } from 'svelte';
  import { QuillMarkdownHost } from './QuillMarkdownHost';

  interface Props {
    value?: string;
    'aria-label'?: string;
    oninput?: (text: string) => void;
    onblur?: () => void;
  }

  let {
    value = '',
    'aria-label': ariaLabel,
    oninput,
    onblur,
  }: Props = $props();

  const editorLabel = $derived(ariaLabel ?? t('ui.editor'));

  let mount: HTMLElement | undefined = $state();
  let host: QuillMarkdownHost | undefined;

  const handlers: {
    oninput?: (text: string) => void;
    onblur?: () => void;
  } = {};

  $effect(() => {
    handlers.oninput = oninput;
    handlers.onblur = onblur;
  });

  onMount(() => {
    if (mount === undefined) return;
    const mountEl = mount;
    host = new QuillMarkdownHost(mountEl, {
      ariaLabel: editorLabel,
      onChange: (text) => handlers.oninput?.(text),
      onBlur: () => handlers.onblur?.(),
    });
    host.setMarkdown(value);

    return () => {
      host?.destroy();
      mountEl.replaceChildren();
      host = undefined;
    };
  });

  $effect(() => {
    host?.setMarkdown(value);
  });
</script>

<div class="markdown-editor">
  <div class="mount" bind:this={mount}></div>
</div>

<style>
  .markdown-editor :global(.ql-toolbar.ql-snow) {
    border: 1px solid var(--color-hairline);
    border-bottom: none;
    border-radius: var(--radius-control) var(--radius-control) 0 0;
    background: var(--color-lift);
  }

  .markdown-editor :global(.ql-container.ql-snow) {
    border: 1px solid var(--color-hairline);
    border-radius: 0 0 var(--radius-control) var(--radius-control);
    background: var(--color-raised);
    color: var(--color-ink);
    font: inherit;
  }

  .markdown-editor :global(.ql-editor) {
    min-height: 12rem;
    line-height: 1.5;
  }

  .markdown-editor :global(.ql-editor.ql-blank::before) {
    color: var(--color-ink-muted);
    font-style: normal;
  }

  .markdown-editor :global(.ql-snow .ql-stroke) {
    stroke: var(--color-ink-muted);
  }

  .markdown-editor :global(.ql-snow .ql-fill) {
    fill: var(--color-ink-muted);
  }

  .markdown-editor :global(.ql-snow .ql-picker) {
    color: var(--color-ink-muted);
  }

  .markdown-editor :global(.ql-snow.ql-toolbar button:hover),
  .markdown-editor :global(.ql-snow.ql-toolbar button.ql-active) {
    color: var(--color-gold);
  }

  .markdown-editor :global(.ql-snow.ql-toolbar button:hover .ql-stroke),
  .markdown-editor :global(.ql-snow.ql-toolbar button.ql-active .ql-stroke) {
    stroke: var(--color-gold);
  }

  .markdown-editor :global(.ql-snow.ql-toolbar button:hover .ql-fill),
  .markdown-editor :global(.ql-snow.ql-toolbar button.ql-active .ql-fill) {
    fill: var(--color-gold);
  }

  .markdown-editor :global(.ql-editor:focus-visible) {
    outline: none;
    box-shadow: var(--focus-ring);
  }
</style>
