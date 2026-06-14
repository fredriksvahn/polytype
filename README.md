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

`--no-keyboard` hides the on-screen keyboard; `--heatmap` colors keys by your
accuracy.

In strict mode the cursor won't advance until you type the correct letter.
Mistyped letters show red; words with an error are underlined; backspace corrects.

## Keybindings

Menu: arrows or `hjkl` to navigate, Enter to start, Ctrl+C to quit.
Test: type to take the test, `Esc` = restart with new words, `Tab` = open the
quick-panel (switch layout/mode), Backspace to correct, Ctrl+C to quit.
Quick-panel: navigate like the menu, Enter applies (new test), `Esc` resumes.
Results: `Tab`/Enter = new test, `Esc` = menu.

Remap any of these in `~/.config/polytype/config.toml`:

    [keys]
    test_restart = "esc"
    test_panel = "tab"
    nav_down = ["down", "j"]

Action names: nav_up, nav_down, nav_prev, nav_next, confirm, quit, test_restart,
test_panel, results_restart, results_menu, panel_cancel. Key strings: named keys
(esc, tab, enter, space, backspace, up/down/left/right), single chars (j, q), and
modifiers (ctrl-, alt-, shift-), e.g. "ctrl-r".

## Layouts

Built in: qwerty, colemak, colemak-dh, colemak-dhm (default), dvorak, workman,
graphite. Add your own by dropping a `.toml` in `~/.config/polytype/layouts/`
(see `assets/layouts/` for the format).

## Config

`~/.config/polytype/config.toml` sets defaults (`target_layout`, `source_layout`,
`show_keyboard`, `show_heatmap`, `wordlist`, ...). Custom wordlists go in
`~/.config/polytype/wordlists/*.txt` (one word per line). Stats persist to
`~/.local/share/polytype/keystats.toml`.
