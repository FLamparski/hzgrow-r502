use hzgrow_r502::{Command, GenImgStatus, MatchStatus, LoadCharResult, LoadCharStatus, Reply, R502};
use serialport::{available_ports, open};
use std::{cell::RefCell, env, time::Duration};

mod pc_utils;
use pc_utils::{SerialReader, SerialWriter};

const DEFAULT_BAUD_RATE: u32 = 57600;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => print_ports(),
        2 => run_test(args[1].as_str(), 0),
        3 => run_test(args[1].as_str(), args[2].parse::<u16>().unwrap()),
        _ => panic!("Usage: pc_fingerprint_match [port_name] [num_char = 0]"),
    };
}

fn print_ports() {
    let ports = available_ports().unwrap();
    for port in ports {
        println!("Available port: {} ({:#?})", port.port_name, port.port_type);
    }
}

fn run_test(port_name: &str, library_index: u16) {
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

    println!("3. Acquiring image");
    print!("Command: {:#?}", Command::GenImg);
    loop {
        match r502.send_command(Command::GenImg) {
            Ok(Reply::GenImg(result)) => match result.confirmation_code {
                GenImgStatus::Success => break,
                GenImgStatus::FingerNotDetected => print!("."),
                GenImgStatus::ImageNotCaptured => print!("!"),
                _ => {}
            },
            Err(e) => panic!("Error: {:#?}", e),
            msg => panic!("Unexpected msg: {:#?}", msg),
        };
    }
    println!();

    println!("4. Checking status - image should be ok");

    println!("Command: {:#?}", Command::ReadSysPara);
    match r502.send_command(Command::ReadSysPara) {
        Ok(Reply::ReadSysPara(result)) => println!(
            "Valid image: {:#?}",
            result.system_parameters.has_valid_image()
        ),
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

    println!("6. Load reference image");

    let cmd = Command::LoadChar {
        buffer: 2,
        index: library_index,
    };
    println!("Command: {:#?}", cmd);
    match r502.send_command(cmd) {
        Ok(Reply::LoadChar(LoadCharResult { confirmation_code: LoadCharStatus::Success, address: _, checksum: _ })) => println!("OK"),
        Ok(Reply::LoadChar(LoadCharResult { confirmation_code, address: _, checksum: _ })) => panic!("Error loading reference data: {:#?}", confirmation_code),
        Err(e) => panic!("Error: {:#?}", e),
        msg => panic!("Unexpected msg: {:#?}", msg),
    };

    println!("7. Match");

    println!("Command: {:#?}", Command::Match);
    match r502.send_command(Command::Match) {
        Ok(Reply::Match(result)) => {
            print!("Confidence value = {}... ", result.match_score);
            match result.confirmation_code {
                MatchStatus::Success => println!("Match successful! *hacker voice* You're in"),
                MatchStatus::NoMatch => println!("No match!"),
                MatchStatus::PacketError => println!("Something bad happened"),
            };
        }
        Err(e) => panic!("Error: {:#?}", e),
        msg => panic!("Unexpected msg: {:#?}", msg),
    };
}
