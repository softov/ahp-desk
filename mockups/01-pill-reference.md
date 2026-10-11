# 01 - doop's pill, for reference

This is what doop ships today, in `packages/tauri-shell`. It is here so the other sketches have something to compare against. It is not a proposal.

The pill is a transparent, always-on-top window with no frame. It shows the bood logo, a label and a status dot for the bound pood.

```text
                desktop
   ####################################
   #  .------------------------------. #
   #  |  (bood)   OpenDoop        ●  | #     ● green: pood is up
   #  '------------------------------' #     ● grey:  pood is down
   ####################################
     ^ 10px transparent padding, click-through, holds the shadow
```

A click opens a panel under the pill. The OS window grows to fit it.

```text
   ####################################################
   #  .------------------------------.                #
   #  |  (bood)   OpenDoop        ●  |                #
   #  '------------------------------'                #
   #  .--------------------------------------------.  #
   #  | Switch                                  ⚙  |  #
   #  |--------------------------------------------|  #
   #  |  ● Local            127.0.0.1:4777  [Open] |  #
   #  |  ● office (HTTP)    10.0.0.5:4777   [Open] |  #
   #  |  ○ dev03 (SSH)      not installed [Install]|  #
   #  |--------------------------------------------|  #
   #  |  [+ Add profile]                           |  #
   #  '--------------------------------------------'  #
   ####################################################
```

`[Open]` opens a separate, normal window at the pood's own web UI.

> In doop, the pill also has a hidden host window behind it. It works around a WebView2 problem on Windows with transparent windows. ahp-desk needs the same workaround if the pill stays a webview.
