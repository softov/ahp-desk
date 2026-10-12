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

const { createEmbedder, unsubscribe } = await import('./embedder.js');

/** The handler the last `listen` call registered for `event`. */
function delivered(event: string): (event: { payload: unknown }) => void {
  const call = listen.mock.calls.find((one) => one[0] === event);
  return call?.[1] as (event: { payload: unknown }) => void;
}

beforeEach(() => {
  vi.clearAllMocks();
  listen.mockResolvedValue(vi.fn());
});

describe('createEmbedder', () => {
  it("reads the window's host and the host list", async () => {
    const embedder = createEmbedder();
    await embedder.host();
    expect(invoke).toHaveBeenCalledWith('window_host');
    await embedder.hosts();
    expect(invoke).toHaveBeenLastCalledWith('hosts');
  });

  it("reads the token of the window's own host", async () => {
    invoke.mockImplementation(async (command: string) =>
      command === 'window_host' ? { id: 'home', name: 'Home', url: 'ws://h' } : 'secret',
    );
    expect(await createEmbedder().token()).toBe('secret');
    expect(invoke).toHaveBeenLastCalledWith('host_token', { id: 'home' });
  });

  it('follows the host list and the statuses', () => {
    const embedder = createEmbedder();
    const hosts = vi.fn();
    const statuses = vi.fn();
    embedder.onHosts(hosts);
    embedder.onStatuses(statuses);
    delivered('hosts-changed')({ payload: [{ id: 'home' }] });
    delivered('hosts-status')({ payload: { home: 'connected' } });
    expect(hosts).toHaveBeenCalledWith([{ id: 'home' }]);
    expect(statuses).toHaveBeenCalledWith({ home: 'connected' });
  });

  it('switches this window to another host, and shows the host list', async () => {
    const embedder = createEmbedder();
    await embedder.switchHost('work');
    expect(invoke).toHaveBeenCalledWith('open_app_window', { hostId: 'work', replace: true });
    await embedder.manageHosts();
    expect(invoke).toHaveBeenLastCalledWith('show_host_list', { add: false });
  });

  it('shows the host list on its Add form', async () => {
    await createEmbedder().manageHosts({ add: true });
    expect(invoke).toHaveBeenLastCalledWith('show_host_list', { add: true });
  });

  it("drives this window's controls", async () => {
    current.onResized.mockResolvedValue(vi.fn());
    const controls = createEmbedder().window;
    await controls?.minimize();
    await controls?.toggleMaximize();
    await controls?.close();
    await controls?.startDragging();
    await controls?.isMaximized();
    const handler = vi.fn();
    controls?.onResized(handler);
    (current.onResized.mock.calls[0]?.[0] as () => void)();
    expect(handler).toHaveBeenCalledOnce();
    for (const control of Object.values(current)) expect(control).toHaveBeenCalledOnce();
  });
});

describe('unsubscribe', () => {
  it('stops a listener once it is set', async () => {
    const unlisten = vi.fn();
    const stop = unsubscribe(Promise.resolve(unlisten));
    await Promise.resolve();
    stop();
    stop();
    expect(unlisten).toHaveBeenCalledOnce();
  });

  it('stops a listener that is set after the stop', async () => {
    const unlisten = vi.fn();
    let set!: (fn: () => void) => void;
    const stop = unsubscribe(new Promise((resolve) => (set = resolve)));
    stop();
    set(unlisten);
    await Promise.resolve();
    await Promise.resolve();
    expect(unlisten).toHaveBeenCalledOnce();
  });
});
