use gpioirq::logs;
use clap::{App, Arg};
use evdev::{Device, Key, InputEventKind};
use std::error::Error;
use std::process::Command;
use tokio::{
    self,
    sync::mpsc,
};
use syslog::{Facility, Formatter3164, BasicLogger};
use log::{LevelFilter, info, warn};
// use tokio::signal::unix::{signal, SignalKind};
// use tokio::time::Duration;

const APPNAME: &'static str = "gpioIrq";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = App::new(APPNAME)
        .version("1.0")
        .author("soporte <soporte@nebulae.com.co>")
        .about("gpio-keys catch signals")
        .arg(
            Arg::with_name("filepath")
                .short("f")
                .long("filepath")
                .value_name("filepath")
                .help("file path of event device, example: /dev/input/event1")
                .takes_value(true))
        .arg(Arg::with_name("logStd")
                .short("l")
                .long("logStd")
                .help("set log to stdout"))
        .get_matches();

    let filename: &str = args.value_of("filepath").unwrap_or("/dev/input/event1");

    let logstd = args.is_present("logStd");
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: APPNAME.to_owned(),
        pid: 0,
    };

    if !logstd {
        let logger = syslog::unix(formatter).expect("could not connect to syslog");
        log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
                .map(|()| log::set_max_level(LevelFilter::Debug))?;
    } else {
        logs::init_std_log()?;
    }
    

    info!("filename: {}", filename);
    let device = Device::open(filename)?;

    

   

    device.supported_keys().map(|keys| {
        log::info!("key: {:?}", keys);
    });

    let mut events = device.into_event_stream()?;

    let (tx, mut rx) = mpsc::channel(32);
    tokio::spawn(async move {
        loop {
            match events.next_event().await {
                Ok(ev) => {
                    if let Err(err) = tx.send(ev).await {
                        println!("event err: {}", err);
                        tx.closed().await;
                        return()
                    }
                },
                Err(err) => {
                    println!("event err: {}", err);
                    continue
                },
            };
        } 
    });

    while let Some(result) = rx.recv().await {
        //println!("{:?}", result);
        let kind = result.kind();
        if let InputEventKind::Key(key) = kind {
            match key {
                Key::KEY_BATTERY => {
                    if result.value() == 0 {
                        info!("BATTERY ON");
                    } else {
                        info!("BATTERY OFF");
                    }
                },
                Key::KEY_WAKEUP => {
                    if result.value() == 1 {
                        info!("IGNITION ON");
                    } else {
                        info!("IGNITION OFF");
                    }
                },
                Key::KEY_PROG1 => {
                    if result.value() == 0 {
                        warn!("KEY_PROG1 ON");
                        Command::new("shutdown").
                            arg("-h").arg("-t 2").
                            spawn().
                            expect("shutdown command failed");
                    } else {
                        info!("KEY_PROG1 OFF");
                    }
                },
                Key::KEY_PROG2 => {
                    if result.value() == 1 {
                        log::info!("ADC alert ON");
                    } else {
                        log::info!("ADC alert OFF");
                    }
                },
                _ => {},
            };
        };

        
    }

    Ok(())
    
}
