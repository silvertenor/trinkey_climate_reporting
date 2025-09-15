use std::error::Error;

use serialport::{self};

#[derive(Debug)]
pub struct Device {
    pub port_name: String,
    pub serial: String,
    pub baud_rate: u32,
}

pub fn get_sensors(vid: u16, pid: u16, baud_rate: u32) -> Result<Vec<Device>, Box<dyn Error>> {
    let mut valid_devices = Vec::new();
    for device in serialport::available_ports()? {
        if cfg!(target_os = "macos") {
            if device.port_name.contains("/dev/tty") {
                continue;
            }
        }
        if let serialport::SerialPortType::UsbPort(info) = device.port_type {
            if info.vid == vid && info.pid == pid {
                let serial = if let Some(serial_no) = info.serial_number {
                    serial_no
                } else {
                    "xxxxxx".to_string()
                };
                valid_devices.push(Device {
                    port_name: device.port_name,
                    serial,
                    baud_rate,
                });
            }
        }
    }
    Ok(valid_devices)
}
