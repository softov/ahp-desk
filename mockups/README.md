# Mockups

ASCII sketches for ahp-desk, to talk over before any plan. What is chosen links to its decision in `.project/decisions/`; the rest is open.

| File | What it sketches |
| --- | --- |
| [01-pill-reference.md](01-pill-reference.md) | doop's pill as it is today, for reference only. |
| [02-hosts.md](02-hosts.md) | The shell's host list: local, remote and SSH hosts. |
| [03-app-window.md](03-app-window.md) | ahpw or ahpapp in a desktop window, with one host or many. |
| [04-bot-on-pill.md](04-bot-on-pill.md) | The bot is the pill, and bots form clusters. Chosen. |
| [05-bot-input.md](05-bot-input.md) | The input that shows on hover or click. Chosen. |
| [06-bot-states.md](06-bot-states.md) | Proposal: how the bot shows what its session is doing. |
| [07-bot-picker.md](07-bot-picker.md) | Proposal: how you choose the bot for the pill. |
| [08-floating-renderer.md](08-floating-renderer.md) | React in a webview, or Rust, for the floating part. |

## Reading the sketches

- The creatures are drawn with ASCII stand-ins. The real bodies are the bood bodies that ahpapp draws (`robot`, `cloud`, `ghost`, `bean` and the others), in the bot's own colour.
- `[ ... ]` is a button. `( )` and `(o)` are radio choices. `▾` opens a list.
- `#` around a frame is the edge of a native OS window. Anything outside it is the desktop.
- A line that starts with `>` is a note on the sketch, not part of the screen.

## Open questions

These are yours to answer. The sketches use one answer as an example only.

1. Does the pill exist when no bot is available, or does the shell hide in the tray only?
2. How many bots can a cluster hold?
3. Is the bot on the pill per host, or one bot across all hosts?
4. Is the pill still the way to reach the host list and settings, as doop's pill is?
