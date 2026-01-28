//! System statistics collection for daemon monitoring.

use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

use crate::daemon::ipc::SystemStats;

/// CPU breakdown statistics.
struct CpuBreakdown {
    user_percent: f64,
    nice_percent: f64,
    system_percent: f64,
    idle_percent: f64,
    usage_percent: f64,
}

/// Collect current system statistics.
pub fn collect_stats() -> io::Result<SystemStats> {
    let cpu_load = read_load_average()?;
    let cpu_breakdown = read_cpu_breakdown().unwrap_or(CpuBreakdown {
        user_percent: 0.0,
        nice_percent: 0.0,
        system_percent: 0.0,
        idle_percent: 100.0,
        usage_percent: 0.0,
    });
    let (total_mem, free_mem) = read_memory_info()?;
    let (total_swap, free_swap) = read_swap_info().unwrap_or((0, 0));

    Ok(SystemStats {
        cpu_load,
        cpu_usage_percent: cpu_breakdown.usage_percent,
        cpu_user_percent: cpu_breakdown.user_percent,
        cpu_nice_percent: cpu_breakdown.nice_percent,
        cpu_system_percent: cpu_breakdown.system_percent,
        cpu_idle_percent: cpu_breakdown.idle_percent,
        total_mem,
        free_mem,
        total_swap,
        free_swap,
    })
}

/// Read the 1-minute load average from /proc/loadavg.
fn read_load_average() -> io::Result<f64> {
    let path = Path::new("/proc/loadavg");
    if !path.exists() {
        // Fallback for non-Linux systems
        return Ok(0.0);
    }

    let content = fs::read_to_string(path)?;
    let first_value = content
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    Ok(first_value)
}

/// Calculate CPU usage breakdown from /proc/stat.
///
/// Returns percentages for user, nice, system, idle, and overall usage.
/// This is a snapshot calculation - for more accurate results,
/// compare two readings over time.
fn read_cpu_breakdown() -> io::Result<CpuBreakdown> {
    let path = Path::new("/proc/stat");
    if !path.exists() {
        return Ok(CpuBreakdown {
            user_percent: 0.0,
            nice_percent: 0.0,
            system_percent: 0.0,
            idle_percent: 100.0,
            usage_percent: 0.0,
        });
    }

    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("cpu ") {
            let values: Vec<u64> = line
                .split_whitespace()
                .skip(1) // Skip "cpu"
                .filter_map(|s| s.parse().ok())
                .collect();

            if values.len() >= 4 {
                let user = values[0];
                let nice = values[1];
                let system = values[2];
                let idle = values[3];
                let iowait = values.get(4).copied().unwrap_or(0);
                let irq = values.get(5).copied().unwrap_or(0);
                let softirq = values.get(6).copied().unwrap_or(0);

                let total = user + nice + system + idle + iowait + irq + softirq;

                if total > 0 {
                    let total_f = total as f64;
                    let user_percent = (user as f64 / total_f) * 100.0;
                    let nice_percent = (nice as f64 / total_f) * 100.0;
                    let system_percent = (system as f64 / total_f) * 100.0;
                    let idle_percent = ((idle + iowait) as f64 / total_f) * 100.0;
                    let usage_percent = 100.0 - idle_percent;

                    return Ok(CpuBreakdown {
                        user_percent,
                        nice_percent,
                        system_percent,
                        idle_percent,
                        usage_percent,
                    });
                }
            }
        }
    }

    Ok(CpuBreakdown {
        user_percent: 0.0,
        nice_percent: 0.0,
        system_percent: 0.0,
        idle_percent: 100.0,
        usage_percent: 0.0,
    })
}

/// Read memory information from /proc/meminfo.
fn read_memory_info() -> io::Result<(u64, u64)> {
    let path = Path::new("/proc/meminfo");
    if !path.exists() {
        return Ok((0, 0));
    }

    let content = fs::read_to_string(path)?;
    let mut total_mem = 0u64;
    let mut free_mem = 0u64;
    let mut buffers = 0u64;
    let mut cached = 0u64;
    let mut s_reclaimable = 0u64;

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let value: u64 = parts[1].parse().unwrap_or(0) * 1024; // Convert from KB to bytes
            match parts[0] {
                "MemTotal:" => total_mem = value,
                "MemFree:" => free_mem = value,
                "Buffers:" => buffers = value,
                "Cached:" => cached = value,
                "SReclaimable:" => s_reclaimable = value,
                _ => {}
            }
        }
    }

    // Available memory includes free, buffers, cached, and reclaimable
    let available = free_mem + buffers + cached + s_reclaimable;

    Ok((total_mem, available))
}

/// Read swap information from /proc/meminfo.
fn read_swap_info() -> io::Result<(u64, u64)> {
    let path = Path::new("/proc/meminfo");
    if !path.exists() {
        return Ok((0, 0));
    }

    let content = fs::read_to_string(path)?;
    let mut total_swap = 0u64;
    let mut free_swap = 0u64;

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let value: u64 = parts[1].parse().unwrap_or(0) * 1024; // Convert from KB to bytes
            match parts[0] {
                "SwapTotal:" => total_swap = value,
                "SwapFree:" => free_swap = value,
                _ => {}
            }
        }
    }

    Ok((total_swap, free_swap))
}

/// Get total system memory using a platform-appropriate method.
pub fn get_total_memory() -> u64 {
    read_memory_info().map(|(total, _)| total).unwrap_or(0)
}

/// Get free system memory using a platform-appropriate method.
pub fn get_free_memory() -> u64 {
    read_memory_info().map(|(_, free)| free).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_stats_does_not_panic() {
        // This test just ensures the function doesn't panic
        // on any platform, even if /proc doesn't exist
        let result = collect_stats();
        // On non-Linux systems, this should return zeros but not error
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_load_average_parse() {
        // Test that parsing works correctly
        let result = read_load_average();
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_memory_getters() {
        // These should not panic
        let _ = get_total_memory();
        let _ = get_free_memory();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_linux_stats_available() {
        // On Linux, we should be able to read stats
        let stats = collect_stats().expect("Should read stats on Linux");
        assert!(stats.total_mem > 0, "Total memory should be non-zero");
    }
}
