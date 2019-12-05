use std::{env, cell::RefCell, time::Duration};
use serialport::{available_ports, open};
use hzgrow_r502::{R502, Command, Reply};

mod pc_utils;
use pc_utils::{SerialReader, SerialWriter};

const DEFAULT_BAUD_RATE: u32 = 57600;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => print_ports(),
        2 => run_test(args[1].as_str()),
        _ => panic!("Usage: pc_fingerprint_search [port_name]"),
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

    println!("1. Verifying password");

    let cmd = Command::VfyPwd { address: 0xffffffff, password: 0x00000000 };
    println!("Command: {:#?}", cmd);
    match r502.send_command(cmd) {
        Ok(Reply::VfyPwd(result)) => println!("Reply: {:#?}", result.confirmation_code),
        Err(e) => println!("Error: {:#?}", e),
        msg => panic!("Unexpected msg: {:#?}", msg),
    };

    println!("2. Checking status - password should be ok");

    let cmd = Command::ReadSysPara { address: 0xffffffff };
    println!("Command: {:#?}", cmd);
    match r502.send_command(cmd) {
        Ok(Reply::ReadSysPara(result)) => println!("Password result: {:#?}", result.system_parameters.password_ok()),
        Err(e) => println!("Error: {:#?}", e),
        msg => panic!("Unexpected msg: {:#?}", msg),
    };

    println!("3. Acquiring image");

    let cmd = Command::GenImg { address: 0xffffffff };
    println!("Command: {:#?}", cmd);
    match r502.send_command(cmd) {
        Ok(reply) => println!("Reply: {:#?}", reply),
        Err(e) => println!("Error: {:#?}", e),
    };
}