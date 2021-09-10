use evdev::{Device};
use std::io::Result;


pub fn new(filename: &str) -> Result<Device> {
    let device = Device::open(filename)?;
    Ok(device)
}