# battui

A live ASCII battery indicator TUI. Project is `battui`; the installed
command is `battery` (see `[[bin]]` in Cargo.toml). Single binary crate,
ratatui 0.29 + crossterm 0.28 + starship-battery.

Renders in an **inline viewport** (`Viewport::Inline`, no alternate screen):
a `VIEWPORT_HEIGHT`-row strip at the cursor, like ordinary command output.
On quit, one final frame is drawn without the key-hints bar and left in the
scrollback, so one-shot use (open, glance, quit) leaves the reading behind.

## Module Responsibilities

```
src/main.rs     — raw-mode guard (incl. panic hook), inline viewport setup,
                  event-driven redraw loop, final hint-less frame on exit
src/app.rs      — App state, key handling, poll timing (POLL_INTERVAL)
src/ui.rs       — all rendering; theme constants live at the top
src/battery.rs  — starship-battery wrapper; never imports ratatui
```

### battery
- `Snapshot` holds display-granularity values only (whole percent, whole
  minutes) and derives `PartialEq` — that equality IS the redraw-on-change
  check, so never add raw float fields to it.
- First battery only; multi-battery machines are out of scope.
- `State::Unknown` (and any future non-exhaustive variants) map to
  `ChargeState::Idle` — on macOS this is "on AC, charge held" (optimized
  charging), shown as "plugged in · not charging".
- No battery at all is a startup error, raised before `ratatui::init()` so
  it prints normally instead of vanishing with the alternate screen.

### app / ui
- **Redraws are event-driven** (dirty flag in `run()`): draw only on input,
  resize, or `poll_battery()` returning true. The battery is re-read every
  `POLL_INTERVAL` (3s) but a poll only reports true when the snapshot
  changed, so idle == zero terminal writes. Don't reintroduce a
  draw-per-tick loop.
- Read errors go to `app.status` (yellow in the bottom bar) and clear on the
  next successful read; the last good snapshot stays on screen.
- Keys: `q`/`Esc`/`Ctrl-C` quit, `r` forces an immediate re-read.
- Raw mode must be undone on every exit path — `main()` installs a panic
  hook for it; keep that if the terminal setup changes.

## Theme

PanEx family (see `~/.claude/themes/panex-tui-style.md`): DarkGray bottom
bar with `key:action` hints, yellow status messages. No frame or title —
the battery art is centered horizontally in the strip; the status line
is centered under the box (nub excluded). Charge shows as 5 discrete buckets (one per
started 20% band), colored by band: Green 5, LightGreen 4, Yellow 3,
orange (Indexed 208) 2, Red 1; unlit buckets DarkGray `░`.

## Running

```sh
cargo run          # debug build (binary is target/debug/battery)
cargo test         # pure-function tests (fill width, h:mm formatting)
cargo clippy       # keep at zero warnings
```

## Verifying TUI changes

Like pathtui, the TUI can't run from a non-tty shell — smoke-test with
`expect` and a pty. The inline viewport changes the recipe in two ways:

1. **Size the pty BEFORE spawn** (`set stty_init`, not a post-spawn stty).
   The inline strip is sized when the Terminal is created; a 0×0 pty at that
   moment means a 0-row viewport that never writes a byte, and no resize
   event fixes it.
2. **Answer the cursor-position query.** `Viewport::Inline` asks `ESC[6n`
   and a bare pty has no terminal emulator to reply, so crossterm times out
   ("cursor position could not be read"). The script must send the reply.

There is also no alternate screen, so don't wait for `1049h`; wait for the
cursor-hide (`?25l`) that precedes the first paint.

```sh
expect -c '
set stty_init "rows 35 columns 110"
spawn ./target/debug/battery
expect -timeout 5 -re "\\\[6n"
send "\x1b\[10;1R"
expect -timeout 8 -re "\\?25l"
expect -timeout 2 "ZZZ_NEVER_MATCHES"
send "q"
expect eof
'
```

Wait with `expect -timeout N "ZZZ_NEVER_MATCHES"`, never `sleep`. ratatui
renders cell diffs — don't grep the raw stream for full strings. Visual
correctness needs a real terminal; say so rather than claiming success.

## Coding Conventions

- Rust edition 2024; `anyhow` for errors (`.context()` at boundaries).
- Pure functions for anything computable (`filled_buckets`,
  `bucket_widths`, `format_hm`) — unit test those; the rendering layer
  stays untested.
- No async runtime, no threads — one cheap IOKit read every few seconds
  happens inline in the event loop.
- `battery.rs` stays free of UI concerns; it never imports ratatui.
