import { render, screen } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import MarkdownEditor from './MarkdownEditor.svelte';

const setMarkdown = vi.fn();
const destroy = vi.fn();
const hostConstruct = vi.fn();

let changeHandler: ((markdown: string) => void) | undefined;
let blurHandler: (() => void) | undefined;

vi.mock('./QuillMarkdownHost', () => ({
  QuillMarkdownHost: class {
    constructor(mount: HTMLElement, options: { ariaLabel: string; onChange: (m: string) => void; onBlur: () => void }) {
      hostConstruct();
      mount.setAttribute('role', 'textbox');
      mount.setAttribute('aria-label', options.ariaLabel);
      changeHandler = options.onChange;
      blurHandler = options.onBlur;
    }

    setMarkdown = setMarkdown;
    destroy = destroy;
  },
}));

describe('MarkdownEditor mount', () => {
  beforeEach(() => {
    hostConstruct.mockClear();
    setMarkdown.mockClear();
    destroy.mockClear();
  });

  it('loads the initial markdown', () => {
    render(MarkdownEditor, {
      value: '## Reflection\n\nA quiet week.\n',
      'aria-label': 'Reflection',
    });
    expect(setMarkdown).toHaveBeenCalledWith('## Reflection\n\nA quiet week.\n');
    expect(screen.getByRole('textbox', { name: 'Reflection' })).toBeInTheDocument();
  });

  it('destroys the host on unmount', () => {
    const { unmount } = render(MarkdownEditor, { value: 'draft' });
    unmount();
    expect(destroy).toHaveBeenCalled();
  });

  it('does not remount when value or handlers change', () => {
    const oninput = vi.fn();
    const { rerender } = render(MarkdownEditor, { value: 'first', oninput });
    expect(hostConstruct).toHaveBeenCalledTimes(1);
    rerender({ value: 'second', oninput: vi.fn(), onblur: vi.fn() });
    expect(hostConstruct).toHaveBeenCalledTimes(1);
  });
});

describe('MarkdownEditor events', () => {
  it('emits markdown when the host reports a user edit', () => {
    const oninput = vi.fn();
    render(MarkdownEditor, { oninput });
    changeHandler?.('## Reflection\n\nMore thoughts.');
    expect(oninput).toHaveBeenCalledWith('## Reflection\n\nMore thoughts.');
  });

  it('reloads when the external value changes', () => {
    const { rerender } = render(MarkdownEditor, { value: 'first' });
    setMarkdown.mockClear();
    rerender({ value: 'second' });
    expect(setMarkdown).toHaveBeenCalledWith('second');
  });

  it('forwards blur to save handlers', () => {
    const onblur = vi.fn();
    render(MarkdownEditor, { onblur });
    blurHandler?.();
    expect(onblur).toHaveBeenCalled();
  });
});
