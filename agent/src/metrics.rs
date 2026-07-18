use anyhow::{Context, Result};
use std::fs;

pub struct RawMetrics {
    pub cpu_usage_percent: f64,
    pub mem_used_bytes: u64,
    pub mem_total_bytes: u64,
    pub load_average_1m: f64,
    pub uptime_seconds: u64,
}

pub fn snapshot() -> Result<RawMetrics> {
    let (mem_used_bytes, mem_total_bytes) = read_mem()?;
    let load_average_1m = read_loadavg()?;
    let uptime_seconds = read_uptime()?;
    let cpu_usage_percent = read_cpu_usage_percent()?;

    Ok(RawMetrics {
        cpu_usage_percent,
        mem_used_bytes,
        mem_total_bytes,
        load_average_1m,
        uptime_seconds,
    })
}

fn read_mem() -> Result<(u64, u64)> {
    let content = fs::read_to_string("/proc/meminfo").context("reading /proc/meminfo")?;
    let mut total_kb = 0u64;
    let mut available_kb = 0u64;

    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            total_kb = parse_kb(rest);
        } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
            available_kb = parse_kb(rest);
        }
    }

    let used_kb = total_kb.saturating_sub(available_kb);
    Ok((used_kb * 1024, total_kb * 1024))
}

fn parse_kb(field: &str) -> u64 {
    field
        .trim()
        .trim_end_matches("kB")
        .trim()
        .parse::<u64>()
        .unwrap_or(0)
}

fn read_loadavg() -> Result<f64> {
    let content = fs::read_to_string("/proc/loadavg").context("reading /proc/loadavg")?;
    let first = content
        .split_whitespace()
        .next()
        .context("unexpected /proc/loadavg format")?;
    Ok(first.parse::<f64>().unwrap_or(0.0))
}

fn read_uptime() -> Result<u64> {
    let content = fs::read_to_string("/proc/uptime").context("reading /proc/uptime")?;
    let first = content
        .split_whitespace()
        .next()
        .context("unexpected /proc/uptime format")?;
    Ok(first.parse::<f64>().unwrap_or(0.0) as u64)
}

fn read_cpu_stat_total() -> Result<(u64, u64)> {
    let content = fs::read_to_string("/proc/stat").context("reading /proc/stat")?;
    let line = content
        .lines()
        .next()
        .context("unexpected /proc/stat format")?;
    let fields: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|f| f.parse::<u64>().ok())
        .collect();

    let idle = *fields.get(3).unwrap_or(&0) + *fields.get(4).unwrap_or(&0);
    let total: u64 = fields.iter().sum();
    Ok((total, idle))
}

fn read_cpu_usage_percent() -> Result<f64> {
    let (total_a, idle_a) = read_cpu_stat_total()?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    let (total_b, idle_b) = read_cpu_stat_total()?;

    let total_delta = total_b.saturating_sub(total_a);
    let idle_delta = idle_b.saturating_sub(idle_a);

    if total_delta == 0 {
        return Ok(0.0);
    }

    let usage = 1.0 - (idle_delta as f64 / total_delta as f64);
    Ok((usage * 100.0).clamp(0.0, 100.0))
}
