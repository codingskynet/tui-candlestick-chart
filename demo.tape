# A VHS tape. See https://github.com/charmbracelet/vhs

Output "target/demo.gif"
Set Theme "Aardvark Blue"
Set Width 1200
Set Height 600
Hide
Type "cargo run --example simple" Enter
Sleep 2s
Show
Set TypingSpeed 100ms
Left 100
Sleep 1s
Hide
Type "q"
