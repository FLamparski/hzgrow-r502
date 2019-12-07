use std::{env, cell::RefCell, time::Duration};
use serialport::{available_ports, open};
use hzgrow_r502::{R502, Command, Reply, GenImgStatus};

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
    let mut r502 = R502::new(writer, reader, 0xffffffff);

    println!("1. Verifying password");

    let cmd = Command::VfyPwd { password: 0x00000000 };
    println!("Command: {:#?}", cmd);
    match r502.send_command(cmd) {
        Ok(Reply::VfyPwd(result)) => println!("Reply: {:#?}", result.confirmation_code),
        Err(e) => panic!("Error: {:#?}", e),
        msg => panic!("Unexpected msg: {:#?}", msg),
    };

    println!("2. Checking status - password should be ok");

    println!("Command: {:#?}", Command::ReadSysPara);
    match r502.send_command(Command::ReadSysPara) {
        Ok(Reply::ReadSysPara(result)) => println!("Password result: {:#?}", result.system_parameters.password_ok()),
        Err(e) => panic!("Error: {:#?}", e),
        msg => panic!("Unexpected msg: {:#?}", msg),
    };

    println!("3. Acquiring image");
    print!("Command: {:#?}", Command::GenImg);
    loop {
        match r502.send_command(Command::GenImg) {
            Ok(Reply::GenImg(result)) => {
                match result.confirmation_code {
                    GenImgStatus::Success => break,
                    GenImgStatus::FingerNotDetected => print!("."),
                    GenImgStatus::ImageNotCaptured => print!("!"),
                    _ => {},
                }
            },
            Err(e) => panic!("Error: {:#?}", e),
            msg => panic!("Unexpected msg: {:#?}", msg),
        };
    }
    println!();

    println!("4. Checking status - image should be ok");

    println!("Command: {:#?}", Command::ReadSysPara);
    match r502.send_command(Command::ReadSysPara) {
        Ok(Reply::ReadSysPara(result)) => println!("Valid image: {:#?}", result.system_parameters.has_valid_image()),
        Err(e) => panic!("Error: {:#?}", e),
        msg => panic!("Unexpected msg: {:#?}", msg),
    };

    println!("5. Process the image into a \"character buffer\"");

    let cmd = Command::Img2Tz { buffer: 1 };
    println!("Command: {:#?}", cmd);
    match r502.send_command(cmd) {
        Ok(Reply::Img2Tz(result)) => println!("Reply: {:#?}", result),
        Err(e) => panic!("Error: {:#?}", e),
        msg => panic!("Unexpected msg: {:#?}", msg),
    };
}