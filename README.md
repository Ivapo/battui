# battui

An ASCII battery indicator for your terminal — a small, live ratatui TUI.
The installed command is `battery`.

```
┌────────────────────────────────────────────┐
│█████████████████████████████████░░░░░░░░░░░│
│█████████████████████████████████░░░░░░░░░░░│▌
│█████████████████████████████████░░░░░░░░░░░│
└────────────────────────────────────────────┘

 93%  3:12 left
```

battui renders **inline** — a short strip right under your prompt, not a
fullscreen app. While it's open it updates live (drain, charge, time
estimates); when you quit, the last reading stays behind in your scrollback
like ordinary command output. So it works equally well as a quick one-shot
glance or left open as a live monitor.

## Features

- Charge bar colored by level: green above 50%, yellow above 20%, red below
- Charging state with ⚡ and time to full; discharging shows time left
- Handles "plugged in, not charging" (macOS optimized/held charge) and full
- Event-driven rendering: polls every 3 s, redraws only when a visible
  value changes — zero terminal writes while idle
- Cross-platform battery readings via
  [starship-battery](https://crates.io/crates/starship-battery)
  (macOS / Linux / Windows)

## Install

```sh
git clone https://github.com/Ivapo/battui
cd battui
cargo install --path .
```

Then:

```sh
battery
```

## Keys

| Key | Action |
|---|---|
| `q` / `Esc` / `Ctrl-C` | quit (leaves the reading in your scrollback) |
| `r` | force an immediate re-read |

## License

[MIT](LICENSE)
