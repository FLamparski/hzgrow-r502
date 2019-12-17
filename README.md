# An incomplete driver for the HZ Grow R502 fingerprint reader module

Uses `embedded-hal` and `arrayvec`. It's not intended to be a complete implementation of the
R502 command set but rather enough for a simple fingerprint verification device.

## Feature support

* Authenticating with the device and querying status
* Searching the fingerprint library
* Verifying selected fingerprints
* Enrolling and deleting fingerprints

For more, see the [projects](https://github.com/FLamparski/hzgrow-r502/projects).

## Examples

Some examples are meant to be run on a full PC rather than an embedded device. Use
a serial to USB converter at 3.3V power and logic levels. I recommend the ESP-PROG.

## Contributing guidelines

If you want to send a PR, please make sure that your changes work on a real R502 (if your changes
modify anything in the driver itself). For issues, do a cursory check to see if a similar issue
has already been filed.

Please follow Rust's [code of conduct](https://www.rust-lang.org/policies/code-of-conduct).
