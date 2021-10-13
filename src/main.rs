

extern crate qwiic_relay_rs;

use qwiic_relay_rs::*;
use std::thread;
use std::time::Duration;

fn main() {
    let _config = QwiicRelayConfig::default();

    let config = QwiicRelayConfig::default();
    let mut qwiic_relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08).expect("Could not init device");

    println!("all off");
    qwiic_relay.set_all_relays_off();
    thread::sleep(Duration::from_secs(2));

    println!("all on");
    qwiic_relay.set_all_relays_on();
    thread::sleep(Duration::from_secs(2));

    println!("all off");
    qwiic_relay.set_all_relays_off();
    thread::sleep(Duration::from_secs(2));
}
