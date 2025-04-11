use std::cmp::max;

/// Calculates the maximum, minimum, and warmup connection limits based on the provided percentage.
pub fn connect_limits(percentage: Option<u32>) -> (u32, u32, u32) {
    let num_cpus = num_cpus::get() as u32;
    let max_connections = max(10, num_cpus * 2);
    let min_connections = num_cpus / 2;
    let warmup_connections = percentage.map_or(0, |perc| {
        (max_connections as f32 * (perc as f32 / 100.0)).ceil() as u32
    });
    
    (max_connections, min_connections, warmup_connections)
}