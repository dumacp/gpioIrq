use std::process::Command;
use std::io::Result;


// pub fn new(filename: &str) -> Result<Device> {
//     let device = Device::open(filename)?;
//     Ok(device)
// }

pub enum SIGNAL {
    ZERO,
    ONE,
}

impl std::string::ToString for SIGNAL {
    fn to_string(&self) -> String {
        match self {
            SIGNAL::ZERO => String::from("0"),
            SIGNAL::ONE => String::from("1"),
            // _ => String::from("0"),
        }
    }
}

pub fn send_signal(led: &str, value: SIGNAL) -> Result<()> {
    Command::new("echo").arg(value.to_string()).arg(">").
        arg(led).
        spawn().map(|_| ())
}