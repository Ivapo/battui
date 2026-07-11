mod app;
mod battery;
mod ui;

use std::io::stdout;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::{Terminal, TerminalOptions, Viewport};

use crate::app::App;

fn main() -> Result<()> {
    // Read the battery before touching the terminal so "no battery detected"
    // prints as a normal error.
    let mut app = App::new()?;

    // Inline viewport: render in a short strip at the cursor like ordinary
    // command output — no alternate screen, and the last frame stays in the
    // scrollback after exit. Raw mode must be undone on every exit path,
    // including panics, or the shell is left unusable.
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        hook(info);
    }));
    enable_raw_mode()?;
    let mut terminal = Terminal::with_options(
        CrosstermBackend::new(stdout()),
        TerminalOptions {
            viewport: Viewport::Inline(ui::VIEWPORT_HEIGHT),
        },
    )?;

    let result = run(&mut terminal, &mut app);

    // One last frame without the key hints, so the scrollback keeps just the
    // battery reading; then park the prompt below the viewport.
    let final_draw = terminal
        .draw(|frame| ui::draw(frame, &app, false))
        .map(|_| ())
        .map_err(anyhow::Error::from);
    disable_raw_mode()?;
    terminal.show_cursor()?;
    println!();
    result.and(final_draw)
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> Result<()> {
    // Draw only when something changed; an idle redraw loop keeps the
    // terminal permanently busy (ratatui writes on every draw call).
    let mut dirty = true;
    while !app.should_quit {
        if app.poll_battery() {
            dirty = true;
        }
        if dirty {
            terminal.draw(|frame| ui::draw(frame, app, true))?;
            dirty = false;
        }
        if event::poll(Duration::from_millis(200))? {
            // Any terminal event (key, resize, focus) may change the screen.
            dirty = true;
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                app.on_key(key);
            }
        }
    }
    Ok(())
}
