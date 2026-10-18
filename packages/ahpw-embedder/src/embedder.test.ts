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

const { createEmbedder, hostUrlOf, unsubscribe } = await import('./embedder.js');

/** A page the desk opened on a host. */
const PAGE = { search: '?host=ws%3A%2F%2F127.0.0.1%3A9187' };

/** The handler the last `listen` call registered for `event`. */
function delivered(event: string): (event: { payload: unknown }) => void {
  const call = listen.mock.calls.find((one) => one[0] === event);
  return call?.[1] as (event: { payload: unknown }) => void;
}

beforeEach(() => {
  vi.clearAllMocks();
  listen.mockResolvedValue(vi.fn());
});

describe('hostUrlOf', () => {
  it("reads the host address the desk put in the page's address", () => {
    expect(hostUrlOf(PAGE)).toBe('ws://127.0.0.1:9187');
  });

  it('stops a page opened with no host', () => {
    expect(() => hostUrlOf({ search: '' })).toThrow('no ?host= address');
  });
});

describe('createEmbedder', () => {
  it('gives the host address synchronously', () => {
    expect(createEmbedder(PAGE).hostUrl).toBe('ws://127.0.0.1:9187');
  });

  it("reads the window's host and the host list", async () => {
    const embedder = createEmbedder(PAGE);
    await embedder.host();
    expect(invoke).toHaveBeenCalledWith('window_host');
    await embedder.hosts();
    expect(invoke).toHaveBeenLastCalledWith('hosts');
  });

  it("reads the token of the window's own host", async () => {
    invoke.mockImplementation(async (command: string) =>
      command === 'window_host' ? { id: 'home', name: 'Home', url: 'ws://h' } : 'secret',
    );
    expect(await createEmbedder(PAGE).token()).toBe('secret');
    expect(invoke).toHaveBeenLastCalledWith('host_token', { id: 'home' });
  });

  it('follows the host list and the statuses', () => {
    const embedder = createEmbedder(PAGE);
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
    const embedder = createEmbedder(PAGE);
    await embedder.switchHost('work');
    expect(invoke).toHaveBeenCalledWith('open_app_window', { hostId: 'work', replace: true });
    await embedder.manageHosts();
    expect(invoke).toHaveBeenLastCalledWith('show_host_list', { add: false });
  });

  it('shows the host list on its Add form', async () => {
    await createEmbedder(PAGE).manageHosts({ add: true });
    expect(invoke).toHaveBeenLastCalledWith('show_host_list', { add: true });
  });

  it("drives this window's controls", async () => {
    current.onResized.mockResolvedValue(vi.fn());
    const controls = createEmbedder(PAGE).window;
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

describe('the window with every host', () => {
  /** The page the desk opens with every host, on the host `work`. */
  const EVERY = { search: '?hosts=every&focus=work' };

  it('says multi and keys its pages under *', () => {
    const embedder = createEmbedder(EVERY);
    expect(embedder.mode).toBe('multi');
    expect(embedder.hostUrl).toBe('*');
  });

  it("reads any host's token", async () => {
    invoke.mockResolvedValue('secret');
    expect(await createEmbedder(EVERY).token('home')).toBe('secret');
    expect(invoke).toHaveBeenLastCalledWith('host_token', { id: 'home' });
  });

  it('gives the host it opened on once, then each host the desk asks for', async () => {
    const forget = vi.fn();
    const embedder = createEmbedder(EVERY, forget);
    const shown = vi.fn();
    embedder.onFocusHost?.(shown);
    await Promise.resolve();
    expect(shown).toHaveBeenCalledWith('work');
    expect(forget).toHaveBeenCalledOnce();
    delivered('focus-host')({ payload: 'home' });
    expect(shown).toHaveBeenLastCalledWith('home');
    embedder.onFocusHost?.(vi.fn());
    expect(forget).toHaveBeenCalledOnce();
  });

  it("shows the host list on a host's form", async () => {
    await createEmbedder(EVERY).manageHosts({ edit: 'home' });
    expect(invoke).toHaveBeenLastCalledWith('show_host_list', { add: false, edit: 'home' });
  });

  it('a window of one host says mono and has no focus to follow', () => {
    const embedder = createEmbedder(PAGE);
    expect(embedder.mode).toBe('mono');
    expect(embedder.onFocusHost).toBeUndefined();
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
