

extern crate qwiic_relay_rs;

use qwiic_relay_rs::*;
use std::thread;
use std::time::Duration;

fn main() {
    let config = QwiicRelayConfig::default();
    let mut qwiic_relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08).expect("Could not init device");
    let version = qwiic_relay.get_version();
    match version {
        Ok(v) => {
            println!("Firmware Version: {}", v);


            println!("all off");
            qwiic_relay.set_all_relays_off().unwrap();
            thread::sleep(Duration::from_secs(2));
        
            println!("all on");
            qwiic_relay.set_all_relays_on().unwrap();
            thread::sleep(Duration::from_secs(2));
        
            println!("all off");
            qwiic_relay.set_all_relays_off().unwrap();
            thread::sleep(Duration::from_secs(2));

            println!("set_relay_on: None");
            qwiic_relay.set_relay_on(None).unwrap();
            thread::sleep(Duration::from_secs(2));

            println!("all off");
            qwiic_relay.set_all_relays_off().unwrap();
            thread::sleep(Duration::from_secs(2));

            println!("set_relay_on: 0");
            qwiic_relay.set_relay_on(Some(0)).unwrap();
            thread::sleep(Duration::from_secs(2));
            
            println!("all off");
            qwiic_relay.set_all_relays_off().unwrap();
            thread::sleep(Duration::from_secs(2));

            println!("set_relay_on: 1");
            qwiic_relay.set_relay_on(Some(1)).unwrap();
            thread::sleep(Duration::from_secs(2));

            println!("all off");
            qwiic_relay.set_all_relays_off().unwrap();
            thread::sleep(Duration::from_secs(2));

            println!("set_relay_on: 2");
            qwiic_relay.set_relay_on(Some(2)).unwrap();
            thread::sleep(Duration::from_secs(2));

            println!("all off");
            qwiic_relay.set_all_relays_off().unwrap();
            thread::sleep(Duration::from_secs(2));

            println!("set_relay_on: 3");
            qwiic_relay.set_relay_on(Some(3)).unwrap();
            thread::sleep(Duration::from_secs(2));
        },
        Err(e) => println!("{:?}", e)
    }




}
