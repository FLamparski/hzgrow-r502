use hzgrow_r502::{Command, GenImgStatus, Reply, R502};
use serialport::{available_ports, open, SerialPort};
use std::{
    cell::RefCell,
    env,
    io::{Read, Write},
    time::Duration,
};

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
    let port = get_configured_serial_port(port_name).unwrap();
    let port_cell = RefCell::new(port);

    let reader = SerialReader(&port_cell);
    let writer = SerialWriter(&port_cell);
    let mut r502 = R502::new(writer, reader, 0xffffffff);

    verify_pwd(&mut r502, 0x00000000).unwrap();

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
    println!("Will enroll a new fingerprint to index {}", index);
    let port = get_configured_serial_port(port_name).unwrap();
    let port_cell = RefCell::new(port);

    let reader = SerialReader(&port_cell);
    let writer = SerialWriter(&port_cell);
    let mut r502 = R502::new(writer, reader, 0xffffffff);

    verify_pwd(&mut r502, 0x00000000).unwrap();

    println!("[1/2] Place finger on reader");
    get_image(&mut r502).unwrap();

    println!("[1/2] Processing the image into a \"character buffer\"");
    process_image(&mut r502, 1).unwrap();

    print!("Now lift your finger and press any key...");
    std::io::stdout().flush().unwrap();
    let mut buf = [0u8];
    std::io::stdin().read(&mut buf).unwrap();
    println!();

    println!("[2/2] Place finger on reader");
    get_image(&mut r502).unwrap();

    println!("[2/2] Processing the image into a \"character buffer\"");
    process_image(&mut r502, 2).unwrap();

    println!("Processing buffers to generate template");
    match r502.send_command(Command::RegModel) {
        Ok(Reply::RegModel(result)) => println!("Reply: {:#?}", result),
        Err(e) => panic!("Error: {:#?}", e),
        msg => panic!("Unexpected msg: {:#?}", msg),
    };

    println!("Saving the template");
    match r502.send_command(Command::Store { index: index, buffer: 1 }) {
        Ok(Reply::Store(result)) => println!("Reply: {:#?}", result),
        Err(e) => panic!("Error: {:#?}", e),
        msg => panic!("Unexpected msg: {:#?}", msg),
    };
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

fn get_image(r502: &mut R502<SerialWriter, SerialReader>) -> Result<(), String> {
    print!("Command: {:#?}", Command::GenImg);
    loop {
        match r502.send_command(Command::GenImg) {
            Ok(Reply::GenImg(result)) => match result.confirmation_code {
                GenImgStatus::Success => break,
                GenImgStatus::FingerNotDetected => print!("."),
                GenImgStatus::ImageNotCaptured => print!("!"),
                _ => {}
            },
            Err(e) => return Err(format!("Error: {:#?}", e)),
            msg => return Err(format!("Unexpected msg: {:#?}", msg)),
        };
        std::io::stdout().flush().unwrap();
    }
    println!();
    return Ok(());
}

fn process_image(r502: &mut R502<SerialWriter, SerialReader>, buffer: u8) -> Result<(), String> {
    let cmd = Command::Img2Tz { buffer: buffer };
    println!("Command: {:#?}", cmd);
    match r502.send_command(cmd) {
        Ok(Reply::Img2Tz(result)) => println!("Reply: {:#?}", result),
        Err(e) => return Err(format!("Error: {:#?}", e)),
        msg => return Err(format!("Unexpected msg: {:#?}", msg)),
    };
    return Ok(());
}
