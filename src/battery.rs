use anyhow::{Context, Result};
use starship_battery::units::ratio::percent;
use starship_battery::units::time::minute;
use starship_battery::{Battery, Manager, State};

/// Everything the UI displays; `PartialEq` drives the redraw-on-change check,
/// so values are pre-rounded to display granularity (whole percent, whole
/// minutes) — raw float readings would differ on every poll.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Snapshot {
    pub percent: u8,
    pub state: ChargeState,
    /// Minutes to full (charging) or to empty (discharging).
    pub minutes: Option<u32>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChargeState {
    Charging,
    Discharging,
    Full,
    Empty,
    /// On AC but not charging (e.g. macOS optimized/held charge).
    Idle,
}

pub struct BatterySource {
    manager: Manager,
    battery: Battery,
}

impl BatterySource {
    pub fn new() -> Result<Self> {
        let manager = Manager::new().context("initializing battery manager")?;
        let battery = manager
            .batteries()
            .context("enumerating batteries")?
            .next()
            .transpose()
            .context("reading battery")?
            .context("no battery detected — battui needs a machine with one")?;
        Ok(Self { manager, battery })
    }

    pub fn read(&mut self) -> Result<Snapshot> {
        self.manager
            .refresh(&mut self.battery)
            .context("refreshing battery state")?;
        Ok(snapshot_of(&self.battery))
    }
}

fn snapshot_of(battery: &Battery) -> Snapshot {
    let state = match battery.state() {
        State::Charging => ChargeState::Charging,
        State::Discharging => ChargeState::Discharging,
        State::Full => ChargeState::Full,
        State::Empty => ChargeState::Empty,
        _ => ChargeState::Idle,
    };
    let time = match state {
        ChargeState::Charging => battery.time_to_full(),
        ChargeState::Discharging => battery.time_to_empty(),
        _ => None,
    };
    Snapshot {
        percent: battery
            .state_of_charge()
            .get::<percent>()
            .round()
            .clamp(0.0, 100.0) as u8,
        state,
        minutes: time.map(|t| t.get::<minute>().round() as u32),
    }
}
