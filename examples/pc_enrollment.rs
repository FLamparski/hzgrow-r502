use hzgrow_r502::{Command, GenImgStatus, MatchStatus, Reply, R502};
use serialport::{available_ports, open};
use std::{cell::RefCell, env, time::Duration};

mod pc_utils;
use pc_utils::{SerialReader, SerialWriter};

const DEFAULT_BAUD_RATE: u32 = 57600;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => print_ports(),
        2 => print_next_template_number(args[1].as_str()),
        3 => enroll_to_id(args[1].as_str(), args[2].parse::<u16>().unwrap()),
        _ => panic!("Usage: pc_enrollment [port_name] [num_char]"),
    };
}

fn print_ports() {
    let ports = available_ports().unwrap();
    for port in ports {
        println!("Available port: {} ({:#?})", port.port_name, port.port_type);
    }
}

fn print_next_template_number(port_name: &str) {
    println!("Using port {}", port_name);
    let mut port = open(port_name).unwrap();
    port.set_baud_rate(DEFAULT_BAUD_RATE).unwrap();
    port.set_timeout(Duration::from_secs(5)).unwrap();

    let port_cell = RefCell::new(port);

    let reader = SerialReader(&port_cell);
    let writer = SerialWriter(&port_cell);
    let mut r502 = R502::new(writer, reader, 0xffffffff);

    println!("1. Verifying password");

    let cmd = Command::VfyPwd {
        password: 0x00000000,
    };
    println!("Command: {:#?}", cmd);
    match r502.send_command(cmd) {
        Ok(Reply::VfyPwd(result)) => println!("Reply: {:#?}", result.confirmation_code),
        Err(e) => panic!("Error: {:#?}", e),
        msg => panic!("Unexpected msg: {:#?}", msg),
    };

    println!("2. Checking status - password should be ok");

    println!("Command: {:#?}", Command::ReadSysPara);
    match r502.send_command(Command::ReadSysPara) {
        Ok(Reply::ReadSysPara(result)) => println!(
            "Password result: {:#?}",
            result.system_parameters.password_ok()
        ),
        Err(e) => panic!("Error: {:#?}", e),
        msg => panic!("Unexpected msg: {:#?}", msg),
    };

    println!("3. Checking next valid template id");

    println!("Command: {:#?}", Command::TemplateNum);
    match r502.send_command(Command::TemplateNum) {
        Ok(Reply::TemplateNum(result)) => println!(
            "[{:#?}] Next valid template number: {}",
            result.confirmation_code, result.template_num,
        ),
        Err(e) => panic!("Error: {:#?}", e),
        msg => panic!("Unexpected msg: {:#?}", msg),
    };
}

fn enroll_to_id(port_name: &str, index: u16) {
    unimplemented!("Not yet implemented");
}
