# mgr

`mgr` is a local web reader for `.cbz` and `.zip` manga volumes, with optional [Mokuro](https://github.com/kha-white/mokuro) OCR support.

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
mgr --generate manga/
mgr --open manga/
```

`--generate` creates or updates a `.mgr.toml` read list in the directory. Directory mode saves reading progress to that file. Archive extensions are matched case-insensitively; supported page images are JPEG, PNG, and WebP.

Mokuro sidecars are detected when they share an archive's filename stem:

```text
volume.cbz
volume.mokuro
```

The same convention applies to `.zip` volumes. The reader serves on `http://localhost:7169` by default.

## Keybindings

Keybindings are configured in the `[keybindings]` table in `config.toml`; they do not have command-line options. This table shows every action and its default binding:

```toml
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

You can include only the bindings you want to change; omitted actions keep their defaults. Set a binding to `""` to disable it. Key names are case-sensitive and follow the browser's `KeyboardEvent.key` values, such as `"ArrowDown"`. Duplicate non-empty keys are rejected.

## Configuration

Configuration files live in `$XDG_CONFIG_HOME/mgr/`. If `XDG_CONFIG_HOME` is unset, `mgr` uses `~/.config/mgr/`.

Options can be set in `config.toml` or passed on the command line. Command-line values override values from the file. For example:

```toml
port = 7169
open = true
zoom = 100
dynamic_colors = false
```

See [Keybindings](#keybindings) for the complete keybinding table and defaults.

| TOML key | CLI option | Default | Description |
| --- | --- | ---: | --- |
| `port` | `-p`, `--port` | `7169` | Local web server port |
| `open` | `-o`, `--open` | `false` | Open the reader in a browser; `--open false` disables it |
| `readlist_file` | `--readlist-file` | `.mgr.toml` | Read-list filename used in directory mode |
| `zoom` | `-z`, `--zoom` | `100` | Initial page width as a percentage |
| `dynamic_colors` | `--dynamic-colors` | `true` | Infer Mokuro text-box colors; set to `false` for black text on white boxes |
| `zoom_min` | `--zoom-min` | `10` | Minimum zoom |
| `zoom_max` | `--zoom-max` | `500` | Maximum zoom |
| `cursor_timeout` | `--cursor-timeout` | `500` ms | Delay before hiding an idle cursor; `0` disables auto-hide |
| `save_debounce` | `--save-debounce` | `200` ms | Delay before saving reading progress after it changes |
| `prefetch_back` | `--prefetch-back` | `6` | Prefetch distance behind the view, in viewport heights |
| `prefetch_forward` | `--prefetch-forward` | `8` | Prefetch distance ahead of the view, in viewport heights |
| `volume_expand_back` | `--volume-expand-back` | `1` | Number of previous volumes to keep expanded |
| `volume_expand_forward` | `--volume-expand-forward` | `1` | Number of following volumes to keep expanded |

`--open` can also be written as `--open true` or `--open false`. `--dynamic-colors` takes an explicit boolean value, such as `--dynamic-colors false`. Zoom values must be positive, and the initial zoom must be between the configured minimum and maximum.

### Custom stylesheet

You can optionally create `styles.css` in the configuration directory. It loads after the built-in stylesheet, so rules with sufficient CSS specificity can override the built-in styles. Changes are picked up when the page is refreshed; restarting `mgr` is not required.

For example, to use a dark page background:

```css
body,
article > section {
    background: #181818;
}
```

Run `mgr --help` for the current command-line usage and options.
