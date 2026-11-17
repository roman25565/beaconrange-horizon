//! BeaconRange for Horizon: beacons give their powers in a radius `multiplier` times the
//! vanilla one. Vanilla (`BeaconBlockEntity.applyEffects`) reaches `10 + 10 * levels` blocks
//! around the beacon horizontally, any height above it and the same distance below it; the game
//! stores that in `range`, and the mod's injection replaces it with [`scale`]`(range)`.
//!
//! The multiplier is read from `config/beaconrange.toml` (beside `server.properties`) the first
//! time a beacon pulses; a missing file is written with the default (4).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const LOG: &str = "[BeaconRange]";
/// Horizon's logger keeps Info/Warn only for targets starting with `horizon` (not this crate's).
const TARGET: &str = "horizon::beaconrange";
pub const CONFIG_FILE: &str = "beaconrange.toml";
pub const DEFAULT_CONFIG: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/beaconrange.toml"));
pub const DEFAULT_MULTIPLIER: f64 = 4.0;
pub const MIN_MULTIPLIER: f64 = 1.0;
pub const MAX_MULTIPLIER: f64 = 16.0;

/// The settings the mod runs with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Config {
    pub multiplier: f64,
}

impl Default for Config {
    fn default() -> Self {
        Config { multiplier: DEFAULT_MULTIPLIER }
    }
}

impl Config {
    /// The radius for a vanilla radius `range`.
    pub fn scale(&self, range: f64) -> f64 {
        range * self.multiplier
    }
}

/// Reads the config text. Unknown keys and bad values are reported and fall back to the default,
/// so a typo never turns beacons off.
pub fn parse(text: &str) -> (Config, Vec<String>) {
    let mut config = Config::default();
    let mut problems = Vec::new();
    for (n, raw) in text.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            problems.push(format!("line {}: expected `key = value`, got {raw:?}", n + 1));
            continue;
        };
        let (key, value) = (key.trim(), value.trim().trim_matches('"'));
        if key != "multiplier" {
            problems.push(format!("line {}: unknown key {key:?}", n + 1));
            continue;
        }
        match value.parse::<f64>() {
            Ok(m) if m.is_finite() && (MIN_MULTIPLIER..=MAX_MULTIPLIER).contains(&m) => config.multiplier = m,
            Ok(m) => problems.push(format!(
                "line {}: multiplier {m} is outside {MIN_MULTIPLIER}..={MAX_MULTIPLIER}, using {DEFAULT_MULTIPLIER}",
                n + 1
            )),
            Err(_) => problems.push(format!(
                "line {}: multiplier {value:?} is not a number, using {DEFAULT_MULTIPLIER}",
                n + 1
            )),
        }
    }
    (config, problems)
}

/// Reads `dir/beaconrange.toml`, writing the default file there first if there is none.
pub fn load(dir: &Path) -> (Config, Vec<String>) {
    let path = dir.join(CONFIG_FILE);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(_) => {
            let mut log = Vec::new();
            if let Err(e) = fs::create_dir_all(dir).and_then(|()| fs::write(&path, DEFAULT_CONFIG)) {
                log.push(format!("could not write {}: {e}", path.display()));
            }
            let (config, mut problems) = parse(DEFAULT_CONFIG);
            log.append(&mut problems);
            return (config, log);
        }
    };
    parse(&text)
}

/// The game's `config/` folder: the working directory's, as Horizon's own
/// `Game::native_config_dir` (beside `server.properties` on a server).
fn config_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_default().join("config")
}

static CONFIG: OnceLock<Config> = OnceLock::new();

/// The config, read the first time it is asked.
pub fn get() -> Config {
    *CONFIG.get_or_init(|| {
        let (config, problems) = load(&config_dir());
        for p in problems {
            log::warn!(target: TARGET, "{LOG} {CONFIG_FILE}: {p}");
        }
        log::info!(target: TARGET, "{LOG} beacon radius x{} (vanilla 20/30/40/50 -> {}/{}/{}/{})",
            config.multiplier,
            config.scale(20.0), config.scale(30.0), config.scale(40.0), config.scale(50.0));
        config
    })
}

/// Called by the injection with the vanilla radius.
pub fn scale(range: f64) -> f64 {
    get().scale(range)
}

/// The vanilla radius for a pyramid of `levels` (1..=4), as the game computes it.
pub fn vanilla_range(levels: i32) -> f64 {
    f64::from(levels * 10 + 10)
}

/// The game's own test (`beacon_effects_on_player`) with a radius: is a player at `player`
/// inside the area of a beacon at block `beacon`? Kept here, equal to the game's, so the tests
/// check the whole rule the injection changes.
pub fn in_range(beacon: [i32; 3], player: [f64; 3], range: f64) -> bool {
    let [x, y, z] = beacon;
    player[0] >= f64::from(x) - range
        && player[0] <= f64::from(x + 1) + range
        && player[2] >= f64::from(z) - range
        && player[2] <= f64::from(z + 1) + range
        && player[1] >= f64::from(y) - range
}

#[cfg(test)]
mod tests;
