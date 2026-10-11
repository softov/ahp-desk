# 06 - What the bot shows (proposal)

The bot shows the state of its own session. bood has moods and actions for this already, so the desktop only has to map a session state to one of them.

The stand-in faces below follow the bood glyph sampler in doop (`scripts/bood-glyphs.mjs`).

```text
     .--.         .--.         .--.         .--.         .--.
   ( - - )      ( o o ) …    ( ^ ^ )      ( ? ? )      ( x x )
    `----'       `----'       `----'       `----'       `----'
    idle        thinking     working      needs you     error
                             (bounces)    (badge, ●)    (red tint)
```

| Session state | Bot | Notes |
| --- | --- | --- |
| No turn running | idle, eyes half shut | It blinks now and then. |
| Turn started, no tool yet | thinking, a `…` beside it | |
| Tool running | working, a small action (`jump`, `wobble`) | The action comes from the body's own `BOOD_ACTIONS`. |
| Waiting on a permission or a question | needs you, an orange badge | A click opens the input on the question. |
| Turn failed | error, red tint | It clears on the next turn. |
| Host down | grey, still | The status badge goes grey. |

## A speech bubble

The last line the bot said, for a few seconds, then it fades. The bot has no frame for text (see 04), so this is where its words go.

```text
   .------------------------------.
   | The build failed on step 3.  |
   '-------------------.  .-------'
                        \/
                       .--.
                     ( o o )●
                      `----'
```

## Coming and going

bood has transits (`beam`, `portal`, `jelly`, `glitch` and others). The bot plays one when the pill shows or hides, or when you change bot.

```text
     .--.        . ' .         .
   ( ^ ^ )  ->  ' .:. '  ->   ' '   ->  (gone)        `portal`
    `----'       ' . '         .
```
