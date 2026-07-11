use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::battery::{BatterySource, Snapshot};

/// How often the battery is re-read while idle. Battery state moves slowly;
/// most polls change nothing and cause no redraw.
const POLL_INTERVAL: Duration = Duration::from_secs(3);

pub struct App {
    source: BatterySource,
    pub snapshot: Snapshot,
    /// Last read error, shown in the bottom bar; cleared by the next good read.
    pub status: Option<String>,
    pub should_quit: bool,
    last_poll: Instant,
}

impl App {
    pub fn new() -> Result<Self> {
        let mut source = BatterySource::new()?;
        let snapshot = source.read()?;
        Ok(Self {
            source,
            snapshot,
            status: None,
            should_quit: false,
            last_poll: Instant::now(),
        })
    }

    /// Re-reads the battery once the poll interval has elapsed; returns true
    /// only when something the UI shows actually changed.
    pub fn poll_battery(&mut self) -> bool {
        if self.last_poll.elapsed() < POLL_INTERVAL {
            return false;
        }
        self.refresh()
    }

    fn refresh(&mut self) -> bool {
        self.last_poll = Instant::now();
        match self.source.read() {
            Ok(snapshot) => {
                let changed = snapshot != self.snapshot || self.status.is_some();
                self.snapshot = snapshot;
                self.status = None;
                changed
            }
            Err(err) => {
                let msg = format!("battery read failed: {err:#}");
                let changed = self.status.as_deref() != Some(msg.as_str());
                self.status = Some(msg);
                changed
            }
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char('r') => {
                self.refresh();
            }
            _ => {}
        }
    }
}
