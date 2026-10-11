# 07 - Choosing the bot for the pill (proposal)

The picker lists the bots that every connected host serves, grouped by host. Choosing one puts it on the pill.

```text
   .------------------------------------------------------------.
   | Choose a bot                                           ×   |
   |------------------------------------------------------------|
   |  soft-sandbox                                              |
   |    .--.                                                    |
   |  ( ^ ^ )  Nimbo     cloud, blue      [x] on the pill       |
   |    `--'                                                    |
   |    [==]                                                    |
   |   [o o]   Unno      robot, green     [ ] on the pill       |
   |    /  \                                                    |
   |                                                            |
   |  This computer                                             |
   |    .-.                                                     |
   |  ( o o)   Unna      ghost, purple    [ ] on the pill       |
   |   `~~~'                                                    |
   |------------------------------------------------------------|
   |  When no bot is on the pill:                               |
   |    (o) show the ahp-desk logo   ( ) hide the pill          |
   |                                                            |
   |  [+ New bot]  -> app window, bot editor                    |
   '------------------------------------------------------------'
```

- Each box is on or off. Two bots on the pill float apart until you drop one on the other to form a cluster (see 04).
- "New bot" opens the editor that ahpapp and ahpw already have. The shell does not have its own.

## When a host stops

```text
   .--.
  ( - - )  Nimbo     soft-sandbox is not reachable
   `--'               (grey on the pill until it comes back)
```
