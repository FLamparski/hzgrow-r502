use std::cell::RefCell;
use serialport::prelude::*;
use embedded_hal::{serial::{Read, Write}};

// We're cheating here and will use the host OS's serial port
// as our UART, and for that we have to implement the read/write
// interfaces from embedded-hal.

pub struct SerialReader<'a>(pub &'a RefCell<Box<dyn SerialPort>>);
pub struct SerialWriter<'a>(pub &'a RefCell<Box<dyn SerialPort>>);

impl Read<u8> for SerialReader<'_> {
    type Error = std::io::Error;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let mut buf: [u8; 1] = [0u8];
        loop {
            match self.0.borrow_mut().read(&mut buf) {
                Ok(n) => if n == 1 {
                    //println!("read: {:02x}", buf[0]);
                    return Ok(buf[0]);
                },
                Err(e) => {
                    println!("Error: {:#?}", e);
                    return Err(nb::Error::from(e));
                },
            };
        }
    }
}

impl Write<u8> for SerialWriter<'_> {
    type Error = std::io::Error;

    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        let buf: [u8; 1] = [word];
        loop {
            match self.0.borrow_mut().write(&buf) {
                Ok(n) => if n == 1 {
                    //println!("write: {:02x}", word);
                    return Ok(());
                },
                Err(e) => {
                    return Err(nb::Error::from(e));
                }
            }
        }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        return match self.0.borrow_mut().flush() {
            Ok(_) => Ok(()),
            Err(e) => Err(nb::Error::from(e)),
        };
    }
}

#[allow(dead_code)]
// This allows us to share code between different PC-based examples.
// There's probably a better way to do it!
fn main() {}
