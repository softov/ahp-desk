# 08 - What draws the floating part

Two ways to draw the pill and its bot. The proposal is to start with React, as doop does, and to move to Rust only if React shows a real problem.

## Option 1 - React in a transparent webview (start here)

```text
   Tauri (Rust)
   ├─ main window       webview: ahpw or ahpapp bundle
   ├─ hidden host       webview: 1x1, not transparent      <- the WebView2 workaround
   └─ pill window       webview: transparent, always on top
                          └─ React: bood body (SVG), the input and its options
```

- bood already exists as SVG with React: ahpapp's `Bood.tsx` and doop's web bood. The pill reuses it.
- The input and its `⋯` options are ordinary HTML.
- Known costs from doop: a transparent webview on Windows needs the hidden host window, and each webview costs memory.

## Option 2 - Rust draws the bot

```text
   Tauri (Rust)
   ├─ main window       webview: ahpw or ahpapp bundle
   └─ pill window       native, no webview
                          └─ Rust: bood paths drawn with tiny-skia or vello
                             the input opens as a small webview on hover or click
```

- No transparent webview, so no WebView2 workaround and less memory while idle.
- The bood paths and moods are data (`bood.ts`), so Rust can read the same data. The animations are code, and they would be written a second time.
- Text input in Rust is hard, so the input stays a webview either way.

## What to measure before choosing

| Measure | How |
| --- | --- |
| Memory with the pill idle | Task Manager on Windows, `ps` on Linux, for each option. |
| CPU while the bot animates | The same, with the `working` state on. |
| Click-through on the transparent area | Click past the bot into the app under it. |
| Linux, X11 and Wayland | Transparency and always-on-top differ between them. |

> Option 2 is a later plan, not part of the first build.
