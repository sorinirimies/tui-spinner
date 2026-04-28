# VHS tape for FluxSpinner — rotating braille dot demo
# Run: vhs examples/vhs/flux-spinner-demo.tape

Output examples/vhs/generated/flux-spinner-demo.gif

Set Theme "Catppuccin Mocha"
Set Width 1400
Set Height 900
Set FontSize 14
Set PlaybackSpeed 1.0

Hide
Type "cargo run --example flux_spinner"
Enter
Sleep 12s
Show
Sleep 8s
Type "q"
Sleep 500ms
