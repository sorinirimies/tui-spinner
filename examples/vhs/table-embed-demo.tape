# VHS tape for the table-embedding demo — spinners inside Table cells
# Run: vhs examples/vhs/table-embed-demo.tape

Output examples/vhs/generated/table-embed-demo.gif

Set Theme "Catppuccin Mocha"
Set Width 1400
Set Height 900
Set FontSize 14
Set PlaybackSpeed 1.0

Hide
Type "cargo run --example table_embed"
Enter
Sleep 12s
Show
Sleep 8s
Type "q"
Sleep 500ms
