# VHS tape for BarSpinner — all 3 pages
# Run: vhs examples/vhs/bar-spinner-demo.tape

Output examples/vhs/generated/bar-spinner-demo.gif

Set Theme "Catppuccin Mocha"
Set Width 1400
Set Height 900
Set FontSize 14
Set PlaybackSpeed 1.0

Hide
Type "cargo run --example bar_spinner"
Enter
Sleep 12s
Show

# Page 1: Braille + mixed styles
Sleep 6s

# Page 2: Symbol styles 4x4
Type "l"
Sleep 6s

# Page 3: Knobs
Type "l"
Sleep 6s

Type "q"
Sleep 500ms
