use clap::{App, Arg};
use evdev::{Device, InputEventKind, Key};
use gpioirq::{gpiokey, logs};
use log::{error, info, warn};
use std::error::Error;
use std::process::Command;
use tokio::{self, sync::mpsc, time};
// use tokio::signal::unix::{signal, SignalKind};
// use tokio::time::{sleep, Duration);

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
                .help("file path of event device, example: /dev/input/event0 (default)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("logStd")
                .short("l")
                .long("logStd")
                .help("set log to stdout"),
        )
        .get_matches();

    let filename: &str = args.value_of("filepath").unwrap_or("/dev/input/event0");

    let logstd = args.is_present("logStd");

    logs::init_std_log(logstd, APPNAME)?;

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
                        error!("event err: {}", err);
                        tx.closed().await;
                        return ();
                    }
                }
                Err(err) => {
                    warn!("event err: {}", err);
                    continue;
                }
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
                }
                Key::KEY_WAKEUP => {
                    if result.value() == 1 {
                        info!("IGNITION ON");
                    } else {
                        info!("IGNITION OFF");
                    }
                }
                Key::KEY_PROG1 => {
                    if result.value() == 0 {
                        warn!("KEY_PROG1 ON");
                        let _ = Command::new("systemctl")
                            .arg("stop")
                            .arg("appfare.service")
                            .spawn()
                            .map_err(|err| {
                                warn!("shutdown appfare command failed, err: {}", err);
                            });
                        time::sleep(time::Duration::from_millis(1000)).await;
                        let _ = Command::new("shutdown")
                            .arg("-h")
                            .arg("now")
                            .spawn()
                            .map_err(|err| {
                                warn!("shutdown command failed, err: {}", err);
                            });
                    } else {
                        info!("KEY_PROG1 OFF");
                    }
                }
                Key::KEY_PROG2 => {
                    if result.value() == 1 {
                        warn!("ADC alert ON");
                        let _ = gpiokey::send_signal(
                            "/sys/class/leds/power-pciusb/brightness",
                            gpiokey::SIGNAL::ZERO,
                        )
                        .map_err(|err| {
                            warn!(r#"shutdown "power-pciusb" command failed, err: {}"#, err);
                        });
                        let _ = gpiokey::send_signal(
                            "/sys/class/leds/reset-usbh1/brightness",
                            gpiokey::SIGNAL::ZERO,
                        )
                        .map_err(|err| {
                            warn!(r#"shutdown "reset-usbh0" command failed, err: {}"#, err);
                        });
                    } else {
                        info!("ADC alert OFF");
                        let _ = gpiokey::send_signal(
                            "/sys/class/leds/reset-usbh1/brightness",
                            gpiokey::SIGNAL::ONE,
                        )
                        .map_err(|err| {
                            warn!(r#"shutdown "reset-usbh0" command failed, err: {}"#, err);
                        });
                        let _ = gpiokey::send_signal(
                            "/sys/class/leds/power-pciusb/brightness",
                            gpiokey::SIGNAL::ONE,
                        )
                        .map_err(|err| {
                            warn!(r#"shutdown "power-pciusb" command failed, err: {}"#, err);
                        });
                    }
                }
                _ => {}
            };
        };
    }

    Ok(())
}
