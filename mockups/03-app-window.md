# 03 - The app window

A normal window with the UI in it. The shell loads the UI bundle and gives it hosts. It does not open a URL that a daemon serves, so the UI does not depend on where it was loaded from.

## One host

ahpapp's single-host mode, or ahpw as it is today.

```text
   ###################################################################
   # ◆ soft-sandbox ▾                                    _   □   ×   #
   #-----------------------------------------------------------------#
   #  Sessions        |  Session: fix the login                      #
   #  ─────────       |  ─────────────────────────────────────────── #
   #  ● fix the login |  ...                                         #
   #  ○ docs pass     |                                              #
   #                  |                                              #
   #                  |  [ Message                          ] [Send] #
   ###################################################################
     ^ title bar drawn by the UI, as doop's web UI draws its own
     ^ "soft-sandbox ▾" switches host, or opens the host list
```

## Many hosts

ahpw's multi-host mode, or ahpapp as it is today.

```text
   ###################################################################
   # ◆ ahp-desk                                          _   □   ×   #
   #-----------------------------------------------------------------#
   #  ● This computer |  Session: fix the login     on soft-sandbox  #
   #    ● notes       |  ─────────────────────────────────────────── #
   #  ● soft-sandbox  |  ...                                         #
   #    ● fix login   |                                              #
   #    ○ docs pass   |                                              #
   #  ○ office        |  [ Message                          ] [Send] #
   ###################################################################
```

## What the shell gives the UI

```text
   shell (Rust)                          UI (ahpw or ahpapp)
   ────────────                          ───────────────────
   host list, tokens from keyring  ───►  hosts: [{ id, name, url, token }]
   one host or many                ───►  mode: 'single' | 'multi'
   window controls                 ◄───  minimize, maximize, close, drag
   open a bot's session            ◄───  from the pill (see 05)
```

> The arrows are the contract to agree first. Both UIs need a way to take hosts from the shell, beside `window.location` (ahpw) and the app's own storage (ahpapp).
