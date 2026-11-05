# mgr

A configurable local web reader for manga with [Mokuro](https://github.com/kha-white/mokuro) support, written in Rust.

## Install

```sh
cargo install --git https://github.com/ank426/mgr
```

Or clone and install from source:

```sh
git clone https://github.com/ank426/mgr
cd mgr
cargo install --path .
```

## Usage

Open one or more volumes:

```sh
mgr volume.cbz
mgr volume1.cbz volume2.cbz volume3.cbz
mgr volume.zip
```

Or open a directory of volumes:

```sh
mgr -g
mgr
```

`-g`/`--generate` creates or updates a `.mgr.toml` read list in the directory. Directory mode saves reading progress to that file. Archive extensions are matched case-insensitively; supported page images are JPEG, PNG, and WebP.

Mokuro files are detected when they share an archive's filename stem:

```text
volume.cbz
volume.mokuro
```

The reader serves on `http://localhost:7169` by default.

```console
$ mgr --help
Usage: mgr [OPTIONS] [PATHS]...

Arguments:
  [PATHS]...  [default: .]

Options:
  -g, --generate
  -p, --port <PORT>                                    [default: 7169]
  -o, --open [<OPEN>]                                  [default: false]
      --readlist-file <READLIST_FILE>                  [default: .mgr.toml]
  -z, --zoom <ZOOM>                                    [default: 100]
      --dynamic-colors <DYNAMIC_COLORS>                [default: true]
      --zoom-min <ZOOM_MIN>                            [default: 10]
      --zoom-max <ZOOM_MAX>                            [default: 500]
      --cursor-timeout <CURSOR_TIMEOUT>                [default: 500]
      --save-debounce <SAVE_DEBOUNCE>                  [default: 200]
      --prefetch-back <PREFETCH_BACK>                  [default: 6]
      --prefetch-forward <PREFETCH_FORWARD>            [default: 8]
      --volume-expand-back <VOLUME_EXPAND_BACK>        [default: 1]
      --volume-expand-forward <VOLUME_EXPAND_FORWARD>  [default: 1]
  -h, --help                                           Print help
```

## Configuration and keybindings

Configuration files live in `$XDG_CONFIG_HOME/mgr/config.toml`. If `XDG_CONFIG_HOME` is unset, `mgr` uses `~/.config/mgr/config.toml`. All settings are optional; this is the complete default configuration:

```toml
port = 7169                  # Local web server port
open = false                 # Open the reader in a browser
readlist_file = ".mgr.toml"  # Read-list filename used in directory mode
zoom = 100                   # Initial page width, as a percentage
dynamic_colors = true        # Infer Mokuro text-box colors
zoom_min = 10                # Minimum zoom percentage
zoom_max = 500               # Maximum zoom percentage
cursor_timeout = 500         # Delay before hiding an idle cursor, in milliseconds (0 disables)
save_debounce = 200          # Delay before saving changed reading progress, in milliseconds
prefetch_back = 6            # Prefetch distance behind the view, in viewport heights
prefetch_forward = 8         # Prefetch distance ahead of the view, in viewport heights
volume_expand_back = 1       # Previous volumes to keep expanded
volume_expand_forward = 1    # Following volumes to keep expanded

[keybindings]
zoom_in = "="           # Increase zoom
zoom_out = "-"          # Decrease zoom
scroll_down = "j"       # Scroll down by half a screen
scroll_up = "k"         # Scroll up by half a screen
previous_volume = "h"   # Go to the previous volume
next_volume = "l"       # Go to the next volume
first_page = "g"        # Go to the first page
last_page = "G"         # Go to the last page
invert_colors = "i"     # Toggle inverted colors
page_progress = "s"     # Toggle page progress
scroll_progress = "d"   # Toggle scroll progress
volume_progress = "S"   # Toggle volume progress
```

Create `config.toml` in that directory and include only the settings you want to change; omitted settings use the defaults shown above. Command-line options override values in the file. Set `dynamic_colors = false` to use black text on white Mokuro boxes instead of inferred colors.

Keybindings are set in the `[keybindings]` table and have no command-line options. Omitted actions keep their defaults; set a binding to `""` to disable it. Key names are case-sensitive and follow the browser's `KeyboardEvent.key` values, such as `"ArrowDown"`. Duplicate non-empty keys are rejected.

### Custom stylesheet

You can create a `styles.css` in the configuration directory. It loads after the built-in stylesheet, so rules with sufficient CSS specificity can override the built-in styles. Changes are picked up when the page is refreshed; restarting `mgr` is not required.
