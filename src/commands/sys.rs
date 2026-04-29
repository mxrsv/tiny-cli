use anyhow::Result;
use sysinfo::{Disks, System};

pub fn run() -> Result<()> {
    let mut system = System::new_all();
    system.refresh_all();

    println!("== System ==");
    if let Some(name) = System::name() {
        println!("OS:        {}", name);
    }
    if let Some(host) = System::host_name() {
        println!("Host:      {}", host);
    }
    let uptime = System::uptime();
    const MAX_REASONABLE_UPTIME_SECS: u64 = 10 * 365 * 86_400;
    if uptime > MAX_REASONABLE_UPTIME_SECS {
        println!("Uptime:    unavailable");
    } else {
        println!("Uptime:    {}", format_duration(uptime));
    }

    println!();
    println!("== CPU ==");
    let cpus = system.cpus();
    println!("Cores:     {}", cpus.len());
    if let Some(first) = cpus.first() {
        println!("Model:     {}", first.brand().trim());
    }

    println!();
    println!("== Memory ==");
    let total = system.total_memory();
    let used = system.used_memory();
    println!("Used:      {}", format_bytes(used));
    println!("Total:     {}", format_bytes(total));
    if total > 0 {
        let pct = (used as f64 / total as f64) * 100.0;
        println!("Usage:     {:.1}%", pct);
    }

    println!();
    println!("== Disks ==");
    let disks = Disks::new_with_refreshed_list();
    if disks.is_empty() {
        println!("(no disks reported)");
    }
    for disk in disks.iter() {
        let total = disk.total_space();
        let avail = disk.available_space();
        let used = total.saturating_sub(avail);
        println!(
            "{:<20} {} used / {} total ({})",
            disk.mount_point().display(),
            format_bytes(used),
            format_bytes(total),
            disk.name().to_string_lossy(),
        );
    }

    Ok(())
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{:.2} {}", value, UNITS[unit])
    }
}

fn format_duration(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let secs = seconds % 60;
    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
