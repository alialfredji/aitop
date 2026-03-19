# VHS demo tape for aitop
# Usage: vhs docs/demo/demo.tape

Output docs/screenshots/demo.gif
Output docs/screenshots/demo.webp

Set Shell "bash"
Set FontFamily "JetBrains Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Padding 20
Set Framerate 15
Set Theme "Catppuccin Mocha"
Set TypingSpeed 40ms
Set WindowBar Colorful
Set WindowBarSize 40
Set BorderRadius 8
Set Margin 20
Set MarginFill "#1e1e2e"
Set CursorBlink false

# Launch aitop
Type "cd /Users/saurabh/Dev/aitop && clear && ./target/release/aitop"
Enter
Sleep 7s

# Switch to Sessions view
Type "s"
Sleep 3s

# Navigate down through sessions
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms

# Open session detail
Enter
Sleep 2s

# Close session detail
Escape
Sleep 1s

# Switch to Models view
Type "m"
Sleep 3s

# Switch to Trends view
Type "t"
Sleep 3s

# Cycle through trend ranges
Type "o"
Sleep 2s
Type "a"
Sleep 2s

# Back to dashboard
Type "d"
Sleep 2s

# Cycle theme
Type "p"
Sleep 2s
Type "p"
Sleep 2s
Type "p"
Sleep 1s

# Back to ember
Type "p"
Sleep 1s
Type "p"
Sleep 1s
Type "p"
Sleep 2s

# Quit
Type "q"
Sleep 500ms
