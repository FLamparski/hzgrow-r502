use std::{env, cell::RefCell, time::Duration};
use serialport::{prelude::*, available_ports, open};
use embedded_hal::{serial::{Read, Write}};
use hzgrow_r502::{R502, Command};

const DEFAULT_BAUD_RATE: u32 = 57600;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => print_ports(),
        2 => run_test(args[1].as_str()),
        _ => panic!("Usage: test_on_pc [port_name]"),
    };
}

fn print_ports() {
    let ports = available_ports().unwrap();
    for port in ports {
        println!("Available port: {} ({:#?})", port.port_name, port.port_type);
    }
}

fn run_test(port_name: &str) {
    println!("Using port {}", port_name);
    let mut port = open(port_name).unwrap();
    port.set_baud_rate(DEFAULT_BAUD_RATE).unwrap();
    port.set_timeout(Duration::from_secs(5)).unwrap();

    let port_cell = RefCell::new(port);

    let reader = SerialReader(&port_cell);
    let writer = SerialWriter(&port_cell);
    let mut r502 = R502::new(writer, reader);

    let check: u16 = 0x01u16 + 0x0003u16 + 0x0fu16;
    println!("expected checksum: {:04x}", check);

    let cmd = Command::ReadSysPara { address: 0xffffffffu32 };
    println!("Command: {:#?}", cmd);
    match r502.send_command(cmd) {
        Ok(reply) => println!("Reply: {:#?}", reply),
        Err(e) => println!("Error: {:#?}", e),
    };
}

// We're cheating here and will use the host OS's serial port
// as our UART, and for that we have to implement the read/write
// interfaces from embedded-hal.

struct SerialReader<'a>(&'a RefCell<Box<dyn SerialPort>>);
struct SerialWriter<'a>(&'a RefCell<Box<dyn SerialPort>>);

impl Read<u8> for SerialReader<'_> {
    type Error = std::io::Error;
    
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let mut buf: [u8; 1] = [0u8];
        loop {
            match self.0.borrow_mut().read(&mut buf) {
                Ok(n) => if n == 1 {
                    println!("read: {:02x}", buf[0]);
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
                    println!("write: {:02x}", word);
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
