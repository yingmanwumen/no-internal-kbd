# No Internal Keyboard

This project disables the internal keyboard of laptop automatically when using external keyboard(s).

It is written in Rust just for fun. A shell script is equivalent.

Running it in a systemd service to enable it automatically.

This tool is only for Linux laptop, and it is really simple so I'm not interested in accomplishing it as it already worked.

## Dependencies

- libevdev-dev
- libudev-dev

## Theory

Use `udev` to

1. collect the information of input devices in `/dev/input/eventX` 
2. monitor the `udev` events, or to say the events of inserting and removing a device

Invoke `ioctl` with signal `EVIOCGRAB` to disable the internal keyboard. And use `evdev` to achieve that for convenience(I don't want to write `unsafe` code).

