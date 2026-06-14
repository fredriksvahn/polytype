# polytype

A layout-agnostic terminal typing trainer. Keep your normal (QWERTY) OS layout;
polytype remaps your keystrokes through the layout you want to learn — Colemak-DHm
by default. Train with free word/timed tests or a graded lesson progression.

## Install

    cargo install --path .

## Usage

    polytype                              # interactive menu
    polytype --layout graphite --time 30  # 30s timed test, train Graphite
    polytype --lesson 1                   # jump into lesson 1
    polytype --source colemak --layout graphite   # OS is Colemak, train Graphite
    polytype --strict --words 15          # stop on error: block until the letter is right
    cat text.txt | polytype --words 50    # use your own words

Keys: arrows to navigate the menu, Enter to start, type to take the test,
`tab` next test, `esc` back to menu, `ctrl-c` quit. `--no-keyboard` hides the
on-screen keyboard; `--heatmap` colors keys by your accuracy.

In strict mode the cursor won't advance until you type the correct letter.
Mistyped letters show red; words with an error are underlined; backspace corrects.

## Layouts

Built in: qwerty, colemak, colemak-dh, colemak-dhm (default), dvorak, workman,
graphite. Add your own by dropping a `.toml` in `~/.config/polytype/layouts/`
(see `assets/layouts/` for the format).

## Config

`~/.config/polytype/config.toml` sets defaults (`target_layout`, `source_layout`,
`show_keyboard`, `show_heatmap`, `wordlist`, ...). Custom wordlists go in
`~/.config/polytype/wordlists/*.txt` (one word per line). Stats persist to
`~/.local/share/polytype/keystats.toml`.
