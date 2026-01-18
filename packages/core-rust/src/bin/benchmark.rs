use core_rust::{excess_gain_excess_loss, max_drawdown, pnl_from_nav, sharpe_ratio};
use std::env;
use std::fs;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    let nav_file = args.get(1).expect("Usage: benchmark <nav_file.csv> <iterations>");
    let iterations: usize = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    // Read NAV data
    let data: Vec<f64> = fs::read_to_string(nav_file)
        .expect("Failed to read NAV file")
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect();

    let hurdle = 0.05;
    let pnl = pnl_from_nav(&data);

    // Warm up
    let _ = sharpe_ratio(&pnl, 252.0, 0.0);
    let _ = max_drawdown(&data);
    let _ = excess_gain_excess_loss(&data, hurdle);

    // Benchmark sharpe_ratio
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = sharpe_ratio(&pnl, 252.0, 0.0);
    }
    let sharpe_time = start.elapsed().as_secs_f64() / iterations as f64 * 1000.0;

    // Benchmark max_drawdown
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = max_drawdown(&data);
    }
    let mdd_time = start.elapsed().as_secs_f64() / iterations as f64 * 1000.0;

    // Benchmark ITH analysis
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = excess_gain_excess_loss(&data, hurdle);
    }
    let ith_time = start.elapsed().as_secs_f64() / iterations as f64 * 1000.0;

    println!("sharpe_ratio_ms:{sharpe_time}");
    println!("max_drawdown_ms:{mdd_time}");
    println!("ith_analysis_ms:{ith_time}");
    println!("total_ms:{}", sharpe_time + mdd_time + ith_time);
}
