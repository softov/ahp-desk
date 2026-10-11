# 05 - The bot's input

Hover on the bot, or click it, and an input shows beside it. The placeholder carries the bot's name, so no name label is drawn anywhere. See [the decision](../.project/decisions/a-bot-is-talked-to-from-an-input-on-hover.md).

## Hover or click

```text
                       .--.
                     ( ^ ^ )●
                      `----'
   .------------------------------------------.
   | Say something to Nimbo...           ⋯  ➤ |
   '------------------------------------------'
```

- The input shows under the bot, or over it when the bot is near the bottom of the screen.
- Hover shows it; moving the mouse away hides it again if nothing was typed.
- A click shows it and keeps it, with the cursor in it. `Esc` or a click outside hides it.
- `Enter` sends a turn into the bot's session. A message sent while a turn is running goes in the session's queue, as in ahpapp.

## After sending

```text
                       .--.
                     ( o o )●…
                      `----'
   .------------------------------------------.
   | you: what failed in the last build?      |
   | Sent                                     |
   |------------------------------------------|
   | Say something to Nimbo...           ⋯  ➤ |
   '------------------------------------------'
```

## The `⋯` options

```text
   .------------------------------------------.
   | Say something to Nimbo...           ⋯  ➤ |
   '------------------------------------------'
   .------------------------------.
   |  💬  Open chat               |   -> app window, on the bot's session
   |  ✎   Edit bot                |   -> app window, bot editor
   |  ⇄   Change bot              |   -> picker (07)
   |  ▦   Layout              ▸   |   -> only in a cluster (04)
   |  ☰   Hosts                   |   -> host list (02)
   |  ⚙   Settings                |
   |  ⏻   Hide                    |
   '------------------------------'
```

## In a cluster

The input belongs to the bot under the mouse. Each bot in a cluster has its own.

```text
     .--.        [==]        .-.
   ( ^ ^ )●     [o o]●     ( o o)●
    `----'       /  \       `~~~'
              .------------------------------------------.
              | Say something to Unno...            ⋯  ➤ |
              '------------------------------------------'
```

## A bot with no session

```text
                       .--.
                     ( - - )●
                      `----'
   .------------------------------------------.
   | Nimbo has no session yet.            ⋯   |
   '------------------------------------------'
```
