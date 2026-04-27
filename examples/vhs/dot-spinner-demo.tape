# VHS tape for DotSpinner — rotating braille dot demo
# Run: vhs examples/vhs/dot-spinner-demo.tape

Output examples/vhs/generated/dot-spinner-demo.gif

Set Theme "Catppuccin Mocha"
Set Width 1400
Set Height 900
Set FontSize 14
Set PlaybackSpeed 1.0

Type "cargo run --example dot_spinner"
Enter

# Wait for compilation and startup
Sleep 10s

# Let all spinners animate through several full rotations
Sleep 6s

# Show the diagonal wave effect
Sleep 4s

# Quit
Type "q"
Sleep 1s
