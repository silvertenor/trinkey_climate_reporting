use std::error::Error;

use climate_sensor::*;
use dotenv::dotenv;
use serialport::SerialPort;
use std::collections::{HashMap, HashSet};
use std::env;
use std::io::{self, BufRead, BufReader, Read};
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
    let port = serialport::new(&device.port_name, 9600)
        .flow_control(serialport::FlowControl::Hardware)
        .open();

    let mut reader = BufReader::new(port.unwrap());

    loop {
        let mut input = String::new();
        match reader.read_line(&mut input) {
            Ok(bytes_read) => {
                if bytes_read > 0 {
                    println!("{input}");
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                // ignore timeout
            }
            Err(e) => {
                eprintln!("Serial read error: {e}");
                break;
            }
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

    let mut known_devices = HashSet::new();
    loop {
        let mut current_devices = HashSet::new();
        if let Ok(devices) = get_sensors(vid, pid, baud_rate) {
            for device in devices {
                current_devices.insert(device.serial.clone());
                if !known_devices.contains(&device.serial) {
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
