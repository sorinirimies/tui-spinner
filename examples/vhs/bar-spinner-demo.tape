# VHS tape for BarSpinner — horizontal + vertical + thickness pages
# Run: vhs examples/vhs/bar-spinner-demo.tape

Output examples/vhs/generated/bar-spinner-demo.gif

Set Theme "Catppuccin Mocha"
Set Width 1400
Set Height 900
Set FontSize 14
Set PlaybackSpeed 1.0

Hide
Type "clear && cargo run --release --example bar_spinner"
Enter
Sleep 5s
Show

# Page 1: Horizontal bars (5 motions)
Sleep 7s

# Page 2: Vertical bars
Type "l"
Sleep 7s

# Page 3: Thickness variations
Type "l"
Sleep 7s

Type "t"
Sleep 2s
Type "T"
Sleep 1s
Type "q"
Sleep 500ms
