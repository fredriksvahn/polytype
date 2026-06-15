# polytype

A layout-agnostic terminal typing trainer. Keep your normal (QWERTY) OS layout;
polytype remaps your keystrokes through the layout you want to learn — Colemak-DHm
by default. Train with free word/timed tests, quotes, or a graded lesson
progression.

## Install

    cargo install --path .

Installs the `polytype` command. Want a shorter name? Add your own alias or
symlink, e.g. `ln -s polytype ~/.cargo/bin/pt`.

## Usage

    polytype                              # interactive menu
    polytype --layout graphite --time 30  # 30s timed test, train Graphite
    polytype --lesson 1                   # jump into lesson 1
    polytype --source colemak --layout graphite   # OS is Colemak, train Graphite
    polytype --strict --words 15          # stop on error: block until the letter is right
    polytype --words 30 --punctuation     # sprinkle punctuation into the words
    polytype --words 30 --numbers         # sprinkle numbers into the words
    polytype --quotes                     # type a random quote/sentence
    polytype --quotes --quote-length long # only long quotes
    polytype --wordlist swedish --words 20 # train on the Swedish word list
    polytype --split                      # space the keyboard halves (split board)
    cat text.txt | polytype --words 50    # use your own words

`--no-keyboard` hides the on-screen keyboard; `--heatmap` colors keys by your
accuracy. `--split` (or `split_keyboard = true` in config) renders the on-screen
keyboard with the two hands spaced apart — handy when you train on a split board.

Words/timed tests can sprinkle punctuation and numbers: pass `--punctuation`
and/or `--numbers`, toggle them in the menu, or set them in config:

    punctuation = true
    numbers = true

Only punctuation that exists on the chosen layout's key grid is used (so it
stays typeable through the remap); numbers pass through unchanged. Lessons are
never decorated.

Quote mode types whole sentences from a bundled list (add your own in
`~/.config/polytype/quotes/*.txt`, one per line). Capitals are kept and typed
with Shift; punctuation not on your layout's grid is stripped. Filter length
with `--quote-length all|short|medium|long` or in the menu. Quotes are not
decorated with punctuation or numbers.

Choose a wordlist with `--wordlist english|swedish` or set `wordlist` in config.
Drop your own lists in `~/.config/polytype/wordlists/<name>.txt` (one word per
line) and select by `<name>`. Swedish keeps å/ä/ö — those pass through the remap,
so they're typeable if your OS layout can produce them. Piping via stdin
overrides the wordlist.

In strict mode the cursor won't advance until you type the correct letter.
Mistyped letters show red; words with an error are underlined; backspace corrects.

## Stats

Pick **Stats** in the menu (or the Tab quick-panel) to see your best and average
WPM, a sparkline of your recent runs, and your weakest fingers for the current
layout. Each finished test is logged to `~/.local/share/polytype/history.csv`.
The Stats screen also shows a per-key **heatmap** of the target layout (green =
accurate, red = weak, dim = untyped) with a legend.

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
graphite, and tarmak1–4 (the transitional steps toward Colemak). Add your own by
dropping a `.toml` in `~/.config/polytype/layouts/` (see `assets/layouts/` for
the format).

## Themes

Set `theme` in config or `--theme <name>` (e.g. `--theme catppuccin-mocha`,
`dracula`, `gruvbox-dark`, `nord`, `rose-pine`, ...). Drop your own in
`~/.config/polytype/themes/<name>.toml` — list the hex slots you want (`bg`,
`fg`, `dim`, `error`, `accent`, `cursor_fg`, `cursor_bg`, `left_hand`,
`right_hand`, `heat_good`, `heat_mid`, `heat_bad`, `heat_unknown`); omitted
slots inherit the default. Combine with **Edit config** to switch live.

Bundled themes: catppuccin-mocha, catppuccin-macchiato, catppuccin-frappe,
catppuccin-latte, dracula, gruvbox-dark, gruvbox-light, nord, rose-pine,
rose-pine-moon, rose-pine-dawn, everforest, solarized-dark, solarized-light,
onedark, kanagawa.

## Config

`~/.config/polytype/config.toml` sets defaults (`target_layout`, `source_layout`,
`show_keyboard`, `show_heatmap`, `split_keyboard`, `wordlist`, ...). Custom wordlists go in
`~/.config/polytype/wordlists/*.txt` (one word per line). Stats persist to
`~/.local/share/polytype/keystats.toml`.

Pick **Edit config** in the menu (or the Tab quick-panel) to open `config.toml`
in `$EDITOR` (`$VISUAL`, else `vi`); on exit, polytype reloads the config so
changes apply immediately.
