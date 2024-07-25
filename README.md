# LKAAS-Kerong

This is an application that interfaces with Kerong's CU16 Board an communicates via MQTT

It utilizes 3 different green threads to post and receive from the following topics:

- `{uid}/events` Topic for changes, when a lock is closed or a package is taken out or put in
- `{uid}/status` Topic for periodic status updates, usually in the range of minutes
- `{uid}/cmd`  Topic to unlock remotely, can be a number from 1 to 255 (application uses a saturating substraction, so it will not crash if an invalid number is received)


## Requirements

- [cross](https://github.com/cross-rs/cross)

## Compiling

This repo includes a makefile, which details the steps to follow to compile and send the executable on the host machine, the executable is sent using SSH

- `cross build --target <TARGET>`
- `scp  ./target/<TARGET>/debug/lkaas-kerong example@127.0.0.1:/usr/local/bin`
