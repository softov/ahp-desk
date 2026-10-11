# 04 - The bot is the pill

Chosen: the bot is the pill. See [the decision](../.project/decisions/the-bot-is-the-pill.md).

When a bot is available, the floating window is only the creature, with a transparent background. There is no frame and no name label. A bot is a host's record at `bot://<slug>`, with a body, a colour, a name and a session.

The stand-in creature below is a `cloud` body. The real one is the SVG body in the bot's colour.

```text
          .--.
        ( ^ ^ )●      ● the status badge (see 06)
         `----'
```

- Drag moves it anywhere on the desktop, as ahpapp's floating cluster moves.
- The name is not drawn. It shows in the input's placeholder when you hover or click (see 05).

## Where it sits

```text
   .----------------------------------------------------------------.
   |                                                                |
   |                                                       .--.     |
   |              your other apps                        ( ^ ^ )●   |
   |                                                      `----'    |
   |                                                                |
   |================================================================|
   | taskbar                                     [tray: ahp-desk]   |
   '----------------------------------------------------------------'
```

> ahpapp keeps the floating position as shares of the window (`float: { x, y }`). On the desktop the same idea is shares of the screen, so the bot keeps its place after a resolution change.

## A cluster of bots

Drop one bot on another and they become a cluster. The cluster moves as one, and you choose how it is laid out.

### On top of one another

```text
          .--.
        ( ^ ^ )●
         `----'
          [==]
         [o o]●
          /  \
          .-.
        ( o o)●
         `~~~'
```

### Side by side

```text
     .--.        [==]        .-.
   ( ^ ^ )●     [o o]●     ( o o)●
    `----'       /  \       `~~~'
```

### Grouped as balloons

The bots pack together in one round shape, touching and a little overlapped, like a bunch of balloons. No strings. Two shapes are sketched below.

#### Circle

The bots sit around one middle, in a ring.

```text
                 .--.
               ( ^ ^ )●
           .-.  `----'  [==]
         ( o o)●       [o o]●
          `~~~'  .--.   /  \
               ( - - )●
                `----'
```

#### Cloud

The bots pile up in a loose lump, bigger at the bottom, the way a cloud is drawn.

```text
               .--.  [==]
             ( ^ ^ )[o o]●
          .-. `----' /  \  .--.
        ( o o)●  .--.     ( - - )●
         `~~~' ( > < )●    `----'
                `----'
```

- Drag any bot of the group to move the whole group.
- A new bot dropped on the group takes the next free place in the shape.

### The cluster's options

A right click on any bot of a cluster, or the `⋯` in its input (see 05):

```text
   .------------------------------.
   |  Layout                      |
   |    (o) On top of one another |
   |    ( ) Side by side          |
   |    ( ) Balloons, circle      |
   |    ( ) Balloons, cloud       |
   |  ──────────────────────────  |
   |  Take Nimbo out              |
   |  Split the cluster           |
   '------------------------------'
```

- "Take Nimbo out" frees one bot, and it floats on its own again.
- A cluster of one bot is a bot on its own.

> Open: how a drag tells "move the group" from "take this bot out" (a modifier key, a long press, or the menu only).

> Open: how many bots a cluster can hold. ahpapp draws at most four (`CLUSTER_MAX`) and puts the rest behind `+N`.
