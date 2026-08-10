import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import TaskCompletionToggle from './TaskCompletionToggle.svelte';

describe('TaskCompletionToggle', () => {
  it('labels an open task with an explicit mark-done action', () => {
    render(TaskCompletionToggle, {
      props: { completed: false, taskTitle: 'File taxes', ontoggle: vi.fn() },
    });
    expect(screen.getByRole('button', { name: 'Mark done: File taxes' })).toHaveTextContent(
      'Mark done',
    );
  });

  it('shows done state and reopens on click', async () => {
    const ontoggle = vi.fn();
    render(TaskCompletionToggle, {
      props: { completed: true, taskTitle: 'File taxes', ontoggle },
    });
    const button = screen.getByRole('button', { name: 'Reopen File taxes' });
    expect(button).toHaveTextContent('Done');
    expect(button).toHaveAttribute('aria-pressed', 'true');
    await userEvent.click(button);
    expect(ontoggle).toHaveBeenCalledOnce();
  });
});
