# @ahpd/desk-client

What a UI calls when it runs inside [ahp-desk](https://github.com/softov/ahp-desk): the host list, the host its window shows, that host's token, and the controls of its frameless window.
In a browser tab, `isDesk()` is false and the UI connects as it always did.

```ts
import { hostToken, isDesk, onHostsChanged, windowControls, windowHost } from '@ahpd/desk-client';

if (isDesk()) {
  const host = await windowHost();
  const token = await hostToken(host.id);
  connect(host.url, token);
  await onHostsChanged((hosts) => renderSwitcher(hosts));
}

titleBar.onmousedown = () => windowControls.startDragging();
```

| Call | What it does |
| --- | --- |
| `isDesk()` | True inside ahp-desk. |
| `hosts()` | Every host in the desk's list, without tokens. |
| `windowHost()` | The host this window shows. |
| `hostToken(id)` | The host's token, or null. A window may only read the token of its own host. |
| `onHostsChanged(handler)` | Calls `handler` with the whole list on each change; resolves to the function that stops it. |
| `openHost(id, { replace })` | Opens the host in its own window, or focuses it; with `replace`, this window switches to it. |
| `showHostList()` | Shows the desk's host list. |
| `hostStatuses()` | Every host's status by id: `connected`, `connecting` or `down`. |
| `onHostStatus(handler)` | Calls `handler` with every host's status on each change; resolves to the function that stops it. |
| `windowControls` | `minimize`, `toggleMaximize`, `close`, `startDragging`, `isMaximized` and `onResized` for this window. |

`@tauri-apps/api` 2 is a peer dependency.
