# 02 - Hosts

The shell keeps the host list. A host is an ahpd, or any AHP server. The UI gets the list from the shell, so ahpw and ahpapp see the same hosts.

## The list

```text
   .-----------------------------------------------------------------.
   | Hosts                                               [+ Add host]|
   |-----------------------------------------------------------------|
   |  ● This computer    ahpd 0.14   127.0.0.1:9187       [Open]  ⋯  |
   |  ● soft-sandbox     ahpd 0.14   wss://sandbox...     [Open]  ⋯  |
   |  ◐ dev03 (SSH)      starting    ssh dev03:2229               ⋯  |
   |  ○ office           down        10.0.0.5:9187       [Retry]  ⋯  |
   |-----------------------------------------------------------------|
   |  Open with   (o) ahpw     ( ) ahpapp                            |
   '-----------------------------------------------------------------'

   ●  connected     ◐  starting or installing     ○  down
   ⋯  Edit, Start, Stop, Remove
```

> "Open with" is here only if both UIs ship in the desktop app. If only one UI ships, this row goes.

## Add a host

```text
   .-----------------------------------------------------------------.
   | Add host                                                        |
   |-----------------------------------------------------------------|
   |  Kind      (o) This computer   ( ) URL   ( ) SSH                |
   |                                                                 |
   |  This computer                                                  |
   |    ahpd     found at ~/.local/bin/ahpd            [Change]      |
   |    Start    [x] when the desktop app starts                     |
   |                                                                 |
   |  URL                                                            |
   |    Address  [ wss://host.example:9187              ]            |
   |    Token    [ ••••••••••••                ]  in the OS keyring  |
   |  SSH                                                            |
   |    Host     [ dev03        ]  Port [ 2229 ]  User [ root  ]     |
   |    Key      [ ~/.ssh/id_ed25519                ] [Browse]       |
   |    ahpd     [x] Install ahpd if it is missing                   |
   |                                                                 |
   |                                     [Cancel]  [Test]  [Save]    |
   '-----------------------------------------------------------------'
```

> doop does the SSH kind with `russh` and a tar push of pood. For ahpd, the install could be `npm i -g` over SSH instead, as ahpd ships on npm. That is a question for the plan.
