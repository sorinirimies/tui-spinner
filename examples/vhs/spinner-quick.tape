# VHS tape for tui-spinner — Quick Preview
# Run: vhs examples/vhs/spinner-quick.tape

Output examples/vhs/generated/spinner-quick.gif

Set Theme "Catppuccin Mocha"
Set Width 1400
Set Height 900
Set FontSize 14
Set PlaybackSpeed 1.5

Hide
Type "clear && cargo run --release --features theme --example spinner"
Enter
Sleep 5s
Show
Sleep 5s
Type "t"
Sleep 2s
Type "T"
Sleep 1s
Type "q"
Sleep 500ms
