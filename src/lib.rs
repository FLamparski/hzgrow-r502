//! **hzgrow-r502** is an embedded-hal driver for the HZ Grow R502 (and likely similar) fingerprint
//! module.
//! 
//! This is a work in progress and does not strive to meet all use cases of the device. However,
//! it should cover the basics, and may possibly become a basis for other drivers for HZ Grow and
//! similar modules, provided that the suppliers of those use a rougly similar API.
//! 
//! ## Example
//! 
//! To authenticate with the R502:
//! ```ignore
//! let (mut tx, mut rx) = serial.split();
//! let mut r502 = R502::new(tx, rx, 0xffffffff);
//! 
//! match r502.send_command(Command::VfyPwd { password: 0x00000000 }) {
//!     Ok(reply) => println!("Reply: {:#?}", reply),
//!     Err(err) => panic!("Error: {:#?}", err),
//! }
//! ```
#![warn(missing_debug_implementations, rust_2018_idioms)]
#![no_std]

mod driver;
mod commands;
mod responses;
mod utils;

pub use crate::driver::R502;
pub use crate::commands::{
    Command,
};
pub use crate::responses::{
    Reply,
    ReadSysParaResult,
    VfyPwdResult,
    SystemParameters,
    PasswordVerificationState,
    GenImgResult,
    GenImgStatus,
    Img2TzResult,
    Img2TzStatus,
    SearchResult,
    SearchStatus,
    LoadCharResult,
    LoadCharStatus,
};
