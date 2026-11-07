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
Type "Q='\x12\x34\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00\x07example\x03com\x00\x00\x01\x00\x01'"
Enter
Type "(sleep 5; minimasque client --proxy 127.0.0.1:4433 --ca minimasque-cert.der --target 1.1.1.1:53 & sleep 1; while true; do printf $Q > /dev/udp/127.0.0.1/5300; sleep 0.2; done) </dev/null >/dev/null 2>&1 &"
Enter
Type "clear"
Enter
Show

Type "minimasque server --tui --listen 127.0.0.1:4433"
Enter
Sleep 14s
Screenshot docs/assets/server.png
Type "q"
Sleep 1s
