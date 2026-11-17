// BeaconRange: the beacon's vanilla range (10 + 10 * pyramid levels) times the configured
// multiplier (config/beaconrange.toml, 4 by default).
fn beaconrange_scale(value: f64) -> f64 {
    beaconrange_core::scale(value)
}
