use hzgrow_r502::{Command, Reply, R502};
use serialport::{available_ports, open, SerialPort};
use std::{
    cell::RefCell,
    env,
    time::Duration,
};

mod pc_utils;
use pc_utils::{SerialReader, SerialWriter};

const DEFAULT_BAUD_RATE: u32 = 57600;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => print_ports(),
        3 => delete_id(args[1].as_str(), args[2].parse::<u16>().unwrap()),
        _ => panic!("Usage: pc_delete [port_name num_char]"),
    };
}

fn print_ports() {
    let ports = available_ports().unwrap();
    for port in ports {
        println!("Available port: {} ({:#?})", port.port_name, port.port_type);
    }
}

fn delete_id(port_name: &str, index: u16) {
    let port = get_configured_serial_port(port_name).unwrap();
    let port_cell = RefCell::new(port);

    let reader = SerialReader(&port_cell);
    let writer = SerialWriter(&port_cell);
    let mut r502 = R502::new(writer, reader, 0xffffffff);

    verify_pwd(&mut r502, 0x00000000).unwrap();

    match r502.send_command(Command::DeletChar { start_index: index, num_to_delete: 1 }) {
        Ok(Reply::DeletChar(result)) => println!("Reply: {:#?}", result),
        Err(e) => panic!("Error: {:#?}", e),
        msg => panic!("Unexpected msg: {:#?}", msg),
    }
}

fn get_configured_serial_port(port_name: &str) -> serialport::Result<Box<dyn SerialPort>> {
    println!("Using port {}", port_name);
    return open(port_name).map(|mut port| {
        port.set_baud_rate(DEFAULT_BAUD_RATE).unwrap();
        port.set_timeout(Duration::from_secs(5)).unwrap();
        return port;
    });
}

fn verify_pwd(r502: &mut R502<SerialWriter, SerialReader>, password: u32) -> Result<(), String> {
    println!("1. Verifying password");

    let cmd = Command::VfyPwd { password: password };
    println!("Command: {:#?}", cmd);
    match r502.send_command(cmd) {
        Ok(Reply::VfyPwd(result)) => println!("Reply: {:#?}", result.confirmation_code),
        Err(e) => return Err(format!("Error: {:#?}", e)),
        msg => return Err(format!("Unexpected msg: {:#?}", msg)),
    };

    println!("2. Checking status - password should be ok");

    println!("Command: {:#?}", Command::ReadSysPara);
    match r502.send_command(Command::ReadSysPara) {
        Ok(Reply::ReadSysPara(result)) => {
            println!(
                "Password result: {:#?}",
                result.system_parameters.password_ok()
            );
            return Ok(());
        }
        Err(e) => return Err(format!("Error: {:#?}", e)),
        msg => return Err(format!("Unexpected msg: {:#?}", msg)),
    };
}
