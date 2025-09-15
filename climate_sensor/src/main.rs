use climate_sensor::*;
use dotenv::dotenv;
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::io::Read;
use std::thread;

fn initialize() -> Result<(u16, u16, u32), Box<dyn Error>> {
    dotenv().ok();
    let vid: u16 = env::var("vid")?.parse()?;
    let pid: u16 = env::var("pid")?.parse()?;
    let baud_rate: u32 = env::var("baud_rate")?.parse()?;
    Ok((vid.into(), pid, baud_rate))
}

fn read(device: Device) {
    println!("{}", device.port_name);
    let mut port = match serialport::new(&device.port_name, device.baud_rate)
        // .flow_control(serialport::FlowControl::Software)
        .open()
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to open serial port {}: {}", device.port_name, e);
            return;
        }
    };

    let mut buf = [0u8; 1024];
    let mut line_buffer = String::new();

    loop {
        match port.read(&mut buf) {
            Ok(n) if n > 0 => {
                // Append newly read bytes to buffer
                line_buffer.push_str(&String::from_utf8_lossy(&buf[..n]));

                // Process complete lines only
                while let Some(pos) = line_buffer.find("\n") {
                    let line = line_buffer[..pos].trim_end_matches(|c| c == '\r' || c == '\n');
                    println!("{line}");
                    line_buffer = line_buffer[pos + 1..].to_string(); // keep remainder
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(e) => {
                eprintln!("Serial read error: {e}");
                break;
            }
            _ => {}
        }
    }
}
fn main() {
    let (vid, pid, baud_rate) = if let Ok(vars) = initialize() {
        vars
    } else {
        eprintln!("Error: could not read dotenv file!");
        return;
    };

    let mut known_devices: HashSet<String> = HashSet::new();
    loop {
        let mut current_devices = HashSet::new();
        if let Ok(devices) = get_sensors(vid, pid, baud_rate) {
            for device in devices {
                current_devices.insert(device.serial.clone());
                if !known_devices.contains(&device.serial) {
                    println!("SPAWNING THREAD for device {device:?}!");
                    thread::spawn(move || read(device));
                }
            }
        }

        for device in current_devices.difference(&known_devices) {
            println!("Device connected: {device}");
        }

        for device in known_devices.difference(&current_devices) {
            eprintln!("Device disconnected: {device}");
        }

        known_devices = current_devices;

        thread::sleep(std::time::Duration::from_secs(5));
    }

    // println!("{:?}", get_sensors());
}
