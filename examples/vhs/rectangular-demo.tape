# VHS tape for RectangularSpinner — Zed / Claude-style bouncing arc demo
# Run: vhs examples/vhs/rectangular-demo.tape

Output examples/vhs/generated/rectangular-demo.gif

Set Theme "Catppuccin Mocha"
Set Width 1400
Set Height 900
Set FontSize 14
Set PlaybackSpeed 1.0

Type "cargo run --example rectangular_spinner"
Enter

# Wait for compilation and startup
Sleep 10s

# Let all spinners animate through several bounces
Sleep 6s

# Show a full bounce cycle
Sleep 4s

# Quit
Type "q"
Sleep 1s
