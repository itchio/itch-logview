# itch-logview

![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)

A small rust CLI for viewing itch logs.

![](https://user-images.githubusercontent.com/7998310/59969502-86056900-954e-11e9-853e-1f07c4629575.png)

And by itch, I mean [the itch.io app](https://github.com/itchio/itch).

## Installation

Clone the repository and cd into it, then:

```bash
cargo install itch-logview
```

## Usage

Run without arguments to get help.

Pass an itch log file to view:

```bash
itch-logview /path/to/itch.txt
```

> Note: in itch v25, the main log is in `$XDG_CONFIG/.config/itch/logs/itch.txt` on Linux, `%APPDATA%/itch/logs/itch.txt` on Windows, `~/Library/Application Support/itch/logs/itch.txt` on macOS.

Use `--follow` (or `-f` for short) to have `tail -f`-like behavior. 

This tool will skip over malformed JSON lines, so if
you pass it a random file chances are there'll just be no output.

## License

itch-logview is released under the MIT License. See the [LICENSE](LICENSE) file for details.

