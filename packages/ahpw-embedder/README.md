# @ahp-desk/ahpw-embedder

ahpw's embedder on the desk.
ahpw owns its embedding contract, `Embedder` in `@ahpd/web/embed`; this package implements it over the desk's Tauri commands and events.

`pnpm build` makes one self-contained script, `dist/ahpw-embedder.js`.
Rust reads it in with `include_str!` and injects it into every app window with `initialization_script`, so `globalThis.ahpwEmbedder` is set before ahpw's own scripts run.
In a browser nothing sets it, and ahpw reads its host from the address bar.

The types come from `@ahpd/web`, linked from `../../../ahpw` until a version with `./embed` is on npm.
Build ahpw once (`pnpm build` in ahpw) before typechecking here.
