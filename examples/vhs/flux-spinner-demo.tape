# VHS tape for FluxSpinner — rotating braille dot demo
# Run: vhs examples/vhs/flux-spinner-demo.tape

Output examples/vhs/generated/flux-spinner-demo.gif

Set Theme "Catppuccin Mocha"
Set Width 1400
Set Height 900
Set FontSize 14
Set PlaybackSpeed 1.0

Type "cargo run --example flux_spinner"
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
