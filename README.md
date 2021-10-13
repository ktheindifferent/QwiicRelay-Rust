# Qwiic Relay I2C library for Rust (WIP)

## Description

This library aims at controlling Qwiic Relays using I2C from Linux. It
primary target is ARM devices such as RaspberryPi or FriendlyARM's NanoPi Neo.
It should nonetheless work on other Linux distributions with access to an I2C
bus.

Currently I only have access to the Quad Solid State Relay for testing purposes. If you have issues with other Qwiic Relays please submit an issue or a pull request.

## How to use library

Add the following line to your cargo.toml:
```
qwiic-relay-rs = { git = "https://github.com/PixelCoda/QwiicLCD-Rust.git", version = "*" }
```

## References

* https://github.com/sparkfun/Qwiic_Relay_Py/blob/main/qwiic_relay.py
* https://github.com/sparkfun/SparkFun_Qwiic_Relay_Arduino_Library/tree/master/src

## License

Released under Apache 2.0.
