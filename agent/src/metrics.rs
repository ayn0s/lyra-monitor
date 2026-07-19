use anyhow::{Context, Result};
use std::fs;

pub struct RawMetrics {
    pub cpu_usage_percent: f64,
    pub mem_used_bytes: u64,
    pub mem_total_bytes: u64,
    pub load_average_1m: f64,
    pub uptime_seconds: u64,
    pub network_rx_bytes_per_sec: u64,
    pub network_tx_bytes_per_sec: u64,
}

pub fn snapshot() -> Result<RawMetrics> {
    let (mem_used_bytes, mem_total_bytes) = read_mem()?;
    let load_average_1m = read_loadavg()?;
    let uptime_seconds = read_uptime()?;
    let (cpu_usage_percent, network_rx_bytes_per_sec, network_tx_bytes_per_sec) =
        read_cpu_and_network()?;

    Ok(RawMetrics {
        cpu_usage_percent,
        mem_used_bytes,
        mem_total_bytes,
        load_average_1m,
        uptime_seconds,
        network_rx_bytes_per_sec,
        network_tx_bytes_per_sec,
    })
}

fn read_mem() -> Result<(u64, u64)> {
    let content = fs::read_to_string("/proc/meminfo").context("reading /proc/meminfo")?;
    Ok(parse_mem_info(&content))
}

fn parse_mem_info(content: &str) -> (u64, u64) {
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
    (used_kb * 1024, total_kb * 1024)
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
    parse_loadavg(&content)
}

fn parse_loadavg(content: &str) -> Result<f64> {
    let first = content
        .split_whitespace()
        .next()
        .context("unexpected /proc/loadavg format")?;
    Ok(first.parse::<f64>().unwrap_or(0.0))
}

fn read_uptime() -> Result<u64> {
    let content = fs::read_to_string("/proc/uptime").context("reading /proc/uptime")?;
    parse_uptime(&content)
}

fn parse_uptime(content: &str) -> Result<u64> {
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
    Ok(parse_cpu_stat_line(line))
}

fn parse_cpu_stat_line(line: &str) -> (u64, u64) {
    let fields: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|f| f.parse::<u64>().ok())
        .collect();

    let idle = *fields.get(3).unwrap_or(&0) + *fields.get(4).unwrap_or(&0);
    let total: u64 = fields.iter().sum();
    (total, idle)
}

fn read_net_dev() -> Result<(u64, u64)> {
    let content = fs::read_to_string("/proc/net/dev").context("reading /proc/net/dev")?;
    Ok(parse_net_dev(&content))
}

fn parse_net_dev(content: &str) -> (u64, u64) {
    let mut rx_total = 0u64;
    let mut tx_total = 0u64;

    for line in content.lines().skip(2) {
        let Some((iface, stats)) = line.split_once(':') else {
            continue;
        };
        if iface.trim() == "lo" {
            continue;
        }

        let fields: Vec<u64> = stats
            .split_whitespace()
            .filter_map(|f| f.parse::<u64>().ok())
            .collect();

        rx_total += fields.first().unwrap_or(&0);
        tx_total += fields.get(8).unwrap_or(&0);
    }

    (rx_total, tx_total)
}

fn read_cpu_and_network() -> Result<(f64, u64, u64)> {
    let (total_a, idle_a) = read_cpu_stat_total()?;
    let (rx_a, tx_a) = read_net_dev()?;
    let started_at = std::time::Instant::now();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let elapsed_secs = started_at.elapsed().as_secs_f64();
    let (total_b, idle_b) = read_cpu_stat_total()?;
    let (rx_b, tx_b) = read_net_dev()?;

    let cpu_usage_percent = cpu_percent_from_samples(total_a, idle_a, total_b, idle_b);
    let rx_bytes_per_sec = bytes_per_sec(rx_a, rx_b, elapsed_secs);
    let tx_bytes_per_sec = bytes_per_sec(tx_a, tx_b, elapsed_secs);

    Ok((cpu_usage_percent, rx_bytes_per_sec, tx_bytes_per_sec))
}

fn bytes_per_sec(before: u64, after: u64, elapsed_secs: f64) -> u64 {
    if elapsed_secs <= 0.0 {
        return 0;
    }
    (after.saturating_sub(before) as f64 / elapsed_secs) as u64
}

fn cpu_percent_from_samples(total_a: u64, idle_a: u64, total_b: u64, idle_b: u64) -> f64 {
    let total_delta = total_b.saturating_sub(total_a);
    let idle_delta = idle_b.saturating_sub(idle_a);

    if total_delta == 0 {
        return 0.0;
    }

    let usage = 1.0 - (idle_delta as f64 / total_delta as f64);
    (usage * 100.0).clamp(0.0, 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_kb_field() {
        assert_eq!(parse_kb("16634802 kB"), 16634802);
        assert_eq!(parse_kb("  1024 kB  "), 1024);
        assert_eq!(parse_kb("not a number"), 0);
    }

    #[test]
    fn parses_mem_info() {
        let content = "MemTotal:       16634802 kB\nMemFree:         1000000 kB\nMemAvailable:    8000000 kB\n";
        let (used_bytes, total_bytes) = parse_mem_info(content);
        assert_eq!(total_bytes, 16634802 * 1024);
        assert_eq!(used_bytes, (16634802 - 8000000) * 1024);
    }

    #[test]
    fn parses_mem_info_missing_fields_defaults_to_zero() {
        let (used_bytes, total_bytes) = parse_mem_info("");
        assert_eq!(used_bytes, 0);
        assert_eq!(total_bytes, 0);
    }

    #[test]
    fn mem_used_never_underflows_when_available_exceeds_total() {
        let content = "MemTotal:       100 kB\nMemAvailable:   200 kB\n";
        let (used_bytes, _total_bytes) = parse_mem_info(content);
        assert_eq!(used_bytes, 0);
    }

    #[test]
    fn parses_loadavg() {
        assert_eq!(parse_loadavg("0.52 0.58 0.59 2/1234 5678\n").unwrap(), 0.52);
    }

    #[test]
    fn parses_loadavg_falls_back_to_zero_on_garbage() {
        assert_eq!(parse_loadavg("garbage 0.58 0.59\n").unwrap(), 0.0);
    }

    #[test]
    fn parses_loadavg_errors_on_empty_content() {
        assert!(parse_loadavg("").is_err());
    }

    #[test]
    fn parses_uptime() {
        assert_eq!(parse_uptime("12345.67 6789.01\n").unwrap(), 12345);
    }

    #[test]
    fn parses_uptime_errors_on_empty_content() {
        assert!(parse_uptime("").is_err());
    }

    #[test]
    fn parses_cpu_stat_line() {
        let (total, idle) = parse_cpu_stat_line("cpu  100 0 200 600 50 0 20 0 0 0");
        assert_eq!(idle, 650);
        assert_eq!(total, 970);
    }

    #[test]
    fn cpu_percent_is_zero_when_fully_idle() {
        assert_eq!(cpu_percent_from_samples(0, 0, 1000, 1000), 0.0);
    }

    #[test]
    fn cpu_percent_is_100_when_fully_busy() {
        assert_eq!(cpu_percent_from_samples(0, 0, 1000, 0), 100.0);
    }

    #[test]
    fn cpu_percent_is_50_for_half_idle_delta() {
        assert_eq!(cpu_percent_from_samples(0, 0, 1000, 500), 50.0);
    }

    #[test]
    fn cpu_percent_is_zero_when_no_time_elapsed() {
        assert_eq!(cpu_percent_from_samples(1000, 500, 1000, 500), 0.0);
    }

    #[test]
    fn parses_net_dev_sums_non_loopback_interfaces() {
        let content = "\
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo:  1000       5    0    0    0     0          0         0     1000       5    0    0    0     0       0          0
  eth0: 20000      50    0    0    0     0          0         0    10000      40    0    0    0     0       0          0
  eth1:  5000      10    0    0    0     0          0         0     2000      15    0    0    0     0       0          0
";
        let (rx, tx) = parse_net_dev(content);
        assert_eq!(rx, 25000);
        assert_eq!(tx, 12000);
    }

    #[test]
    fn parses_net_dev_empty_content() {
        assert_eq!(parse_net_dev(""), (0, 0));
    }

    #[test]
    fn bytes_per_sec_computes_rate() {
        assert_eq!(bytes_per_sec(1000, 2000, 0.5), 2000);
    }

    #[test]
    fn bytes_per_sec_is_zero_on_zero_elapsed() {
        assert_eq!(bytes_per_sec(1000, 2000, 0.0), 0);
    }

    #[test]
    fn bytes_per_sec_never_underflows_on_counter_reset() {
        assert_eq!(bytes_per_sec(2000, 1000, 0.5), 0);
    }
}
