# Qwiic Relay I2C library for Rust (WIP)

## Description

This library aims at controlling Qwiic Relays using I2C from Linux. It
primary target is ARM devices such as RaspberryPi or FriendlyARM's NanoPi Neo.
It should nonetheless work on other Linux distributions with access to an I2C
bus.

Currently I only have access to the Quad Solid State Relay for testing purposes. If you have issues with other Qwiic Relays please submit an issue or a pull request.

Roadmap:
* Map relay commands and addresses to structs (DONE)
* Ability to toggle all relays on/off (DONE)
* Ability to toggle individual relays on/off (DONE)
* Ability to read relay status (DONE)
* Ability to check firmware version (DONE)
* Ability to change relay hardware address (WIP)

## How to use library

Add the following line to your cargo.toml:
```
qwiic-relay-rs = "0.1.1"
```

Or for the most recent commit on the master branch use:
```
qwiic-relay-rs = { git = "https://github.com/PixelCoda/QwiicRelay-Rust.git", version = "*" }
```

Example:
```rust


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

            println!("set_relay_on: 1");
            qwiic_relay.set_relay_on(Some(1)).unwrap();
            thread::sleep(Duration::from_secs(2));

            println!("get_relay_state: 1");
            let relay_one_state = qwiic_relay.get_relay_state(Some(1)).unwrap();
            if relay_one_state {
                println!("relay 1 is on!");
            }
            thread::sleep(Duration::from_secs(2));
            

            println!("set_relay_off: 1");
            qwiic_relay.set_relay_off(Some(1)).unwrap();
            thread::sleep(Duration::from_secs(2));

            println!("set_relay_on: 2");
            qwiic_relay.set_relay_on(Some(2)).unwrap();
            thread::sleep(Duration::from_secs(2));

            println!("get_relay_state: 2");
            let relay_one_state = qwiic_relay.get_relay_state(Some(2)).unwrap();
            if relay_one_state {
                println!("relay 2 is on!");
            }
            thread::sleep(Duration::from_secs(2));

            println!("set_relay_off: 2");
            qwiic_relay.set_relay_off(Some(2)).unwrap();
            thread::sleep(Duration::from_secs(2));

            println!("set_relay_on: 3");
            qwiic_relay.set_relay_on(Some(3)).unwrap();
            thread::sleep(Duration::from_secs(2));

            println!("get_relay_state: 3");
            let relay_one_state = qwiic_relay.get_relay_state(Some(3)).unwrap();
            if relay_one_state {
                println!("relay 3 is on!");
            }
            thread::sleep(Duration::from_secs(2));

            println!("set_relay_off: 3");
            qwiic_relay.set_relay_off(Some(3)).unwrap();
            thread::sleep(Duration::from_secs(2));

            println!("set_relay_on: 4");
            qwiic_relay.set_relay_on(Some(4)).unwrap();
            thread::sleep(Duration::from_secs(2));

            println!("get_relay_state: 4");
            let relay_one_state = qwiic_relay.get_relay_state(Some(4)).unwrap();
            if relay_one_state {
                println!("relay 4 is on!");
            }
            thread::sleep(Duration::from_secs(2));
    
            println!("set_relay_off: 4");
            qwiic_relay.set_relay_off(Some(4)).unwrap();
            thread::sleep(Duration::from_secs(2));
        },
        Err(e) => println!("{:?}", e)
    }
}
```

## References

* https://github.com/sparkfun/Qwiic_Relay_Py/blob/main/qwiic_relay.py
* https://github.com/sparkfun/SparkFun_Qwiic_Relay_Arduino_Library/tree/master/src

## License

Released under Apache 2.0.

# Support and follow my work by:

#### Buying my dope NTFs:
 * https://opensea.io/accounts/PixelCoda

#### Checking out my Github:
 * https://github.com/PixelCoda

#### Following my facebook page:
 * https://www.facebook.com/pixelcoda/

#### Subscribing to my Patreon:
 * https://www.patreon.com/calebsmith_pixelcoda

#### Or donating crypto:
 * ADA:    addr1vyjsx8zthl5fks8xjsf6fkrqqsxr4f5tprfwux5zsnz862glwmyr3
 * BTC:    3BCj9kYsqyENKU5YgrtHgdQh5iA7zxeJJi
 * MANA:   0x10DFc66F881226f2B91D552e0Cf7231C1e409114
 * SHIB:   0xdE897d5b511A66276E9B91A8040F2592553e6c28


