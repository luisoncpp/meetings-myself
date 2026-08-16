import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import type { LibraryView } from '../../domain';
import LinkModal from './LinkModal.svelte';

const sampleLibrary: LibraryView = {
  values: [{ id: 'v1', title: 'Health', archived: false }],
  goals: [
    { id: 'g1', title: 'Run a marathon', achieved: false, targetDate: null, archived: false },
    { id: 'g2', title: 'Eat better', achieved: false, targetDate: null, archived: false },
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
  tasks: [
    {
      id: 't1',
      title: 'Buy running shoes',
      state: 'open',
      importance: 'high',
      urgency: 'high',
      deadline: null,
      overdue: false,
      archived: false,
      oneOff: true,
    },
  ],
  associations: [
    {
      id: 'a1',
      left: { kind: 'goal', id: 'g1' },
      right: { kind: 'habit', id: 'h1' },
      lifecycle: 'active',
      createdAt: '2026-08-01',
    },
  ],
};

describe('LinkModal tab display', () => {
  it('shows only Goal tab for Value and selects it', () => {
    const onlink = vi.fn().mockResolvedValue(undefined);
    render(LinkModal, {
      fromEnd: { kind: 'value', id: 'v1' },
      fromTitle: 'Health',
      view: sampleLibrary,
      onlink,
      onclose: vi.fn(),
    });
    expect(screen.getByText('Run a marathon')).toBeInTheDocument();
    expect(screen.getByText('Eat better')).toBeInTheDocument();
  });
});

describe('LinkModal tab ordering', () => {
  it('orders tabs as Goal, Habit for Task and defaults to Goal', async () => {
    render(LinkModal, {
      fromEnd: { kind: 'task', id: 't1' },
      fromTitle: 'Buy running shoes',
      view: sampleLibrary,
      onlink: vi.fn().mockResolvedValue(undefined),
      onclose: vi.fn(),
    });
    const tabs = screen.getAllByRole('tab');
    expect(tabs.map((t) => t.textContent?.trim().toLowerCase())).toEqual(['goal', 'habit']);
    expect(tabs[0]).toHaveAttribute('aria-selected', 'true');
    expect(screen.getByText('Run a marathon')).toBeInTheDocument();
    await userEvent.click(tabs[1]!);
    expect(tabs[1]).toHaveAttribute('aria-selected', 'true');
    expect(screen.getByText('Morning jogging')).toBeInTheDocument();
  });
});

describe('LinkModal candidate filtering', () => {
  it('excludes already linked entities from candidate list', () => {
    render(LinkModal, {
      fromEnd: { kind: 'goal', id: 'g1' },
      fromTitle: 'Run a marathon',
      view: sampleLibrary,
      onlink: vi.fn().mockResolvedValue(undefined),
      onclose: vi.fn(),
    });
    const tabs = screen.getAllByRole('tab');
    expect(tabs.map((t) => t.textContent?.trim().toLowerCase())).toEqual(['habit', 'value', 'task']);
    expect(screen.queryByText('Morning jogging')).not.toBeInTheDocument();
  });
});

describe('LinkModal action completion', () => {
  it('links an item and closes the modal', async () => {
    const onlink = vi.fn().mockResolvedValue(undefined);
    const onclose = vi.fn();
    render(LinkModal, {
      fromEnd: { kind: 'task', id: 't1' },
      fromTitle: 'Buy running shoes',
      view: sampleLibrary,
      onlink,
      onclose,
    });
    const linkButtons = screen.getAllByRole('button', { name: /^link$/i });
    await userEvent.click(linkButtons[0]!);
    expect(onlink).toHaveBeenCalledWith({ kind: 'goal', id: 'g1' });
    expect(onclose).toHaveBeenCalled();
  });
});
