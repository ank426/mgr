# mgr

Local web reader for `.cbz` manga volumes with [Mokuro](https://github.com/kha-white/mokuro) OCR support, written in Rust.

## Features

- Read one or more `.cbz` volumes in a browser, served on `localhost`.
- Generate a directory read list with saved reading position.
- Show selectable Japanese text from adjacent `.mokuro` files (goes well with [Yomitan](https://github.com/themoeway/yomitan)).
- Keyboard controls for navigation, zoom, inversion, and progress.
- Extensive cli and file configuration.

## Install

```bash
cargo install --git https://github.com/ank426/mgr
```

Or clone and install:

```bash
git clone https://github.com/ank426/mgr
cd mgr
cargo install --path .
```

## Usage

Open one or more volumes:

```bash
mgr volume.cbz
mgr volume1.cbz volume2.cbz volume3.cbz
```

Open a directory of volumes:

```bash
mgr --generate manga/
mgr --open manga/
```

`--generate` creates a `.mgr.toml` read list from the directory's CBZ files, in natural sort order. Directory mode saves reading progress to this file.

Mokuro files are detected automatically when they share a CBZ volume's name:

```text
volume.cbz
volume.mokuro
```

The reader is available at `http://localhost:7169` by default.

## Controls

| Key | Action |
| --- | --- |
| `j` / `k` | Scroll down / up half a screen |
| `h` / `l` | Previous / next volume |
| `g` / `G` | First / last page |
| `=` / `-` | Zoom in / out |
| `i` | Invert colors |
| `s` | Page progress overlay |
| `d` | Scroll progress overlay |
| `S` | Volume progress overlay |

## Configuration

Options may be passed on the command line or placed in `$XDG_CONFIG_HOME/mgr/config.toml`:

```toml
port = 7169
open = true
zoom = 100
```

```text
> mgr --help
Usage: mgr [OPTIONS] [PATHS]...

Arguments:
  [PATHS]...  [default: .]

Options:
  -g, --generate
  -p, --port <PORT>                              [default: 7169]
  -o, --open [<OPEN>]                            [default: false]
      --readlist-file <READLIST_FILE>            [default: .mgr.toml]
  -z, --zoom <ZOOM>                              [default: 100]
      --zoom-min <ZOOM_MIN>                      [default: 10]
      --zoom-max <ZOOM_MAX>                      [default: 500]
      --cursor-timeout <CURSOR_TIMEOUT>          [default: 500]
      --save-debounce <SAVE_DEBOUNCE>            [default: 200]
      --prefetch-back <PREFETCH_BACK>            [default: 6]
      --prefetch-forward <PREFETCH_FORWARD>      [default: 8]
      --volume-expand-back <VOLUME_EXPAND_BACK>  [default: 1]
      --volume-expand-forward <VOLUME_EXPAND_FORWARD>
                                                    [default: 1]
  -h, --help                                     Print help
```
