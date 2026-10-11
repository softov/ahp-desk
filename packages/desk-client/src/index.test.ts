import { beforeEach, describe, expect, it, vi } from 'vitest';

const invoke = vi.fn();
const listen = vi.fn();
const current = {
  minimize: vi.fn(),
  toggleMaximize: vi.fn(),
  close: vi.fn(),
  startDragging: vi.fn(),
  isMaximized: vi.fn(),
  onResized: vi.fn(),
};
vi.mock('@tauri-apps/api/core', () => ({ invoke }));
vi.mock('@tauri-apps/api/event', () => ({ listen }));
vi.mock('@tauri-apps/api/window', () => ({ getCurrentWindow: () => current }));

const client = await import('./index.js');

beforeEach(() => {
  vi.clearAllMocks();
});

describe('desk-client', () => {
  it('reads the host list', async () => {
    invoke.mockResolvedValue([]);
    await client.hosts();
    expect(invoke).toHaveBeenCalledWith('hosts');
  });

  it("reads the window's host", async () => {
    await client.windowHost();
    expect(invoke).toHaveBeenCalledWith('window_host');
  });

  it("reads a host's token by its id", async () => {
    await client.hostToken('home');
    expect(invoke).toHaveBeenCalledWith('host_token', { id: 'home' });
  });

  it('hands the handler the list of each change, and returns the unlisten', async () => {
    const unlisten = vi.fn();
    listen.mockResolvedValue(unlisten);
    const handler = vi.fn();
    expect(await client.onHostsChanged(handler)).toBe(unlisten);
    expect(listen.mock.calls[0]?.[0]).toBe('hosts-changed');
    const deliver = listen.mock.calls[0]?.[1] as (event: {
      payload: unknown;
    }) => void;
    deliver({ payload: [{ id: 'home' }] });
    expect(handler).toHaveBeenCalledWith([{ id: 'home' }]);
  });

  it('opens a host in a new window, or in this one with replace', async () => {
    await client.openHost('home');
    expect(invoke).toHaveBeenCalledWith('open_app_window', {
      hostId: 'home',
      replace: false,
    });
    await client.openHost('work', { replace: true });
    expect(invoke).toHaveBeenLastCalledWith('open_app_window', {
      hostId: 'work',
      replace: true,
    });
  });

  it('shows the host list', async () => {
    await client.showHostList();
    expect(invoke).toHaveBeenCalledWith('show_host_list');
  });

  it("drives this window's controls", async () => {
    await client.windowControls.minimize();
    await client.windowControls.toggleMaximize();
    await client.windowControls.close();
    await client.windowControls.startDragging();
    await client.windowControls.isMaximized();
    const handler = vi.fn();
    await client.windowControls.onResized(handler);
    (current.onResized.mock.calls[0]?.[0] as () => void)();
    expect(handler).toHaveBeenCalledOnce();
    for (const control of Object.values(current))
      expect(control).toHaveBeenCalledOnce();
  });

  it('reads every host status, and follows them', async () => {
    await client.hostStatuses();
    expect(invoke).toHaveBeenCalledWith('host_statuses');
    const handler = vi.fn();
    await client.onHostStatus(handler);
    expect(listen.mock.calls[0]?.[0]).toBe('hosts-status');
    const deliver = listen.mock.calls[0]?.[1] as (event: {
      payload: unknown;
    }) => void;
    deliver({ payload: { home: 'connected' } });
    expect(handler).toHaveBeenCalledWith({ home: 'connected' });
  });

  it('knows a browser tab is not the desk', () => {
    expect(client.isDesk()).toBe(false);
  });
});
