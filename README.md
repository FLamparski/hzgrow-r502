# An incomplete driver for the HZ Grow R502 fingerprint reader module

Uses `embedded-hal` and `arrayvec`. It's not intended to be a complete implementation of the
R502 command set but rather enough for a simple fingerprint verification device.

## Progress

- [ ] **VerifyPassword** Performs a handshake with the device
- [x] **ReadSysPara** Reads system status information
- [ ] ...

## Examples

Some examples are meant to be run on a full PC rather than an embedded device. Use
a serial to USB converter.