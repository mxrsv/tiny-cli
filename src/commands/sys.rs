use anyhow::Result;
use sysinfo::{Disks, System};

use crate::util::{format_bytes, format_duration};

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

