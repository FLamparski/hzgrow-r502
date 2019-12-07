# An incomplete driver for the HZ Grow R502 fingerprint reader module

Uses `embedded-hal` and `arrayvec`. It's not intended to be a complete implementation of the
R502 command set but rather enough for a simple fingerprint verification device.

## Progress

- [x] **VerifyPassword** Performs a handshake with the device
- [x] **ReadSysPara** Reads system status information
- [x] **GenImg** Captures a fingerprint image
- [x] **Img2Tz** Converts an image into an internal fingerprint representation
- [x] **Search** Searches for fingerprints in the library

For more, see the [projects](https://github.com/FLamparski/hzgrow-r502/projects).

## Examples

Some examples are meant to be run on a full PC rather than an embedded device. Use
a serial to USB converter.