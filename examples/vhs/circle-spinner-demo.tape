# VHS tape for CircleSpinner — Clockwise ↻ / Counter-Clockwise ↺, radii 2–8
# Run: vhs examples/vhs/circle-spinner-demo.tape

Output examples/vhs/generated/circle-spinner-demo.gif

Set Theme "Catppuccin Mocha"
Set Width 1400
Set Height 900
Set FontSize 14
Set PlaybackSpeed 1.0

Hide
Type "cargo run --example circle_spinner"
Enter
Sleep 12s
Show
Sleep 8s
Type "q"
Sleep 500ms
