import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import type { LibraryView } from '../../../domain';
import AssociationTags from './AssociationTags.svelte';

const sampleLibrary: LibraryView = {
  values: [{ id: 'v1', title: 'Health', archived: false }],
  goals: [
    { id: 'g1', title: 'Run a marathon', achieved: false, targetDate: null, archived: false },
  ],
  habits: [
    {
      id: 'h1',
      title: 'Morning jogging',
      cadence: { kind: 'everyDay' },
      strength: 'established',
      pinned: true,
      archived: false,
    },
  ],
  tasks: [],
  associations: [
    {
      id: 'a1',
      left: { kind: 'value', id: 'v1' },
      right: { kind: 'goal', id: 'g1' },
      lifecycle: 'active',
      createdAt: '2026-08-01',
    },
    {
      id: 'a2',
      left: { kind: 'goal', id: 'g1' },
      right: { kind: 'habit', id: 'h1' },
      lifecycle: 'active',
      createdAt: '2026-08-01',
    },
  ],
};

describe('AssociationTags rendering', () => {
  it('renders tags with tooltip indicating kind and title', () => {
    render(AssociationTags, {
      end: { kind: 'goal', id: 'g1' },
      view: sampleLibrary,
      onunlink: vi.fn(),
      onopenLink: vi.fn(),
    });
    expect(screen.getByText('Health')).toBeInTheDocument();
    expect(screen.getByText('Morning jogging')).toBeInTheDocument();
    expect(screen.getByText('Health').closest('.tag')).toHaveAttribute('title', 'Value: Health');
    expect(screen.getByText('Morning jogging').closest('.tag')).toHaveAttribute('title', 'Habit: Morning jogging');
  });
});

describe('AssociationTags unlink', () => {
  it('unlinks when the X button is clicked', async () => {
    const onunlink = vi.fn();
    render(AssociationTags, {
      end: { kind: 'goal', id: 'g1' },
      view: sampleLibrary,
      onunlink,
      onopenLink: vi.fn(),
    });
    const unlinkButtons = screen.getAllByRole('button', { name: /^remove link/i });
    expect(unlinkButtons).toHaveLength(2);
    await userEvent.click(unlinkButtons[0]!);
    expect(onunlink).toHaveBeenCalledWith('a1');
  });
});

describe('AssociationTags link trigger', () => {
  it('triggers onopenLink when Link to button is clicked', async () => {
    const onopenLink = vi.fn();
    render(AssociationTags, {
      end: { kind: 'goal', id: 'g1' },
      view: sampleLibrary,
      onunlink: vi.fn(),
      onopenLink,
    });
    await userEvent.click(screen.getByRole('button', { name: /^link to…$/i }));
    expect(onopenLink).toHaveBeenCalled();
  });
});
