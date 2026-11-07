Output docs/assets/migration.gif

Set Shell bash
Set FontSize 15
Set Width 1280
Set Height 720
Set Padding 16
Set Theme "Catppuccin Mocha"
Set TypingSpeed 40ms

Hide
Type "cd $(mktemp -d)"
Enter
Type "minimasque server --listen 127.0.0.1:4434 --cert-out proxy.der >/dev/null 2>&1 &"
Enter
Type "Q='\x12\x34\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00\x07example\x03com\x00\x00\x01\x00\x01'"
Enter
Type "(sleep 3; while true; do printf $Q > /dev/udp/127.0.0.1/5300; sleep 0.2; done) >/dev/null 2>&1 &"
Enter
Sleep 1s
Type "clear"
Enter
Show

Type "minimasque client --tui --proxy 127.0.0.1:4434 --ca proxy.der --target 1.1.1.1:53"
Sleep 500ms
Enter
Sleep 7s
Type "m"
Sleep 5s
Type "m"
Sleep 6s
Type "q"
Sleep 1s
