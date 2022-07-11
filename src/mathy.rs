pub const SIMULATION_PRECISION: f64 = 1e9;

pub fn round_to(x: f64, precision: f64) -> f64 {
    (x * precision).round() / precision
}
