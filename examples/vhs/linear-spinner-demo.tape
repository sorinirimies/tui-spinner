# VHS tape for LinearSpinner — all styles × orientation × direction
# Run: vhs examples/vhs/linear-spinner-demo.tape

Output examples/vhs/generated/linear-spinner-demo.gif

Set Theme "Catppuccin Mocha"
Set Width 1400
Set Height 900
Set FontSize 14
Set PlaybackSpeed 1.0

Hide
Type "clear && cargo run --release --features theme --example linear_spinner"
Enter
Sleep 5s
Show
Sleep 8s
Type "t"
Sleep 2s
Type "T"
Sleep 1s
Type "q"
Sleep 500ms
