# An incomplete driver for the HZ Grow R502 fingerprint reader module

Uses `embedded-hal` and `arrayvec`. It's not intended to be a complete implementation of the
R502 command set but rather enough for a simple fingerprint verification device.

## Progress

- [ ] **VerifyPassword** Performs a handshake with the device
- [ ] **ReadSysPara** Reads system status information
- [ ] ...