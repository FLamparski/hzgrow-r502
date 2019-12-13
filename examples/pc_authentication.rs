use hzgrow_r502::{Command, R502};
use serialport::{available_ports, open};
use std::{cell::RefCell, env, time::Duration};

mod pc_utils;
use pc_utils::{SerialReader, SerialWriter};

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
    let mut r502 = R502::new(writer, reader, 0xffffffff);

    println!("1. Checking status");

    let cmd = Command::ReadSysPara;
    println!("Command: {:#?}", cmd);
    match r502.send_command(cmd) {
        Ok(reply) => println!("Reply: {:#?}", reply),
        Err(e) => println!("Error: {:#?}", e),
    };

    println!("2. Verifying password");

    let cmd = Command::VfyPwd {
        password: 0x00000000,
    };
    println!("Command: {:#?}", cmd);
    match r502.send_command(cmd) {
        Ok(reply) => println!("Reply: {:#?}", reply),
        Err(e) => println!("Error: {:#?}", e),
    };

    println!("3. Checking status again - password should be ok");

    let cmd = Command::ReadSysPara;
    println!("Command: {:#?}", cmd);
    match r502.send_command(cmd) {
        Ok(reply) => println!("Reply: {:#?}", reply),
        Err(e) => println!("Error: {:#?}", e),
    };
}
