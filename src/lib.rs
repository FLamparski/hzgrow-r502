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
//! ```
//! # use embedded_hal::serial::{Read, Write};
//! use hzgrow_r502::{R502, Command, Reply};
//! # struct TestTx;
//! # struct TestRx(usize);
//! #
//! # impl Write<u8> for TestTx {
//! #     type Error = ();
//! #     fn write(&mut self, _word: u8) -> nb::Result<(), Self::Error> {
//! #         return Ok(());
//! #     }
//! #     fn flush(&mut self) -> nb::Result<(), Self::Error> {
//! #         return Ok(());
//! #     }
//! # }
//! #
//! # const res_data: &[u8] = &[ 0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0x07, 0x00, 0x03, 0x00, 0x00, 0x0a ];
//! #
//! # impl Read<u8> for TestRx {
//! #     type Error = ();
//! #     fn read(&mut self) -> nb::Result<u8, Self::Error> {
//! #         let word = res_data[self.0];
//! #         self.0 += 1;
//! #         return Ok(word);
//! #     }
//! # }
//! # let mut rx = TestRx(0);
//! # let mut tx = TestTx;
//!
//! // Obtain tx, rx from some serial port implementation
//! let mut r502 = R502::new(tx, rx, 0xffffffff);
//! match r502.send_command(Command::VfyPwd { password: 0x00000000 }) {
//!     Ok(Reply::VfyPwd(result)) => println!("Status: {:#?}", result.confirmation_code),
//!     Err(error) => panic!("Error: {:#?}", error),
//!     _ => {},
//! }
//! ```
//!
//! For more examples, see [the `examples` directory](https://github.com/FLamparski/hzgrow-r502/tree/master/examples).
#![warn(missing_debug_implementations, rust_2018_idioms)]
#![no_std]

mod commands;
mod driver;
mod responses;
mod utils;

pub use crate::commands::Command;
pub use crate::driver::R502;
pub use crate::responses::{
    GenImgResult, GenImgStatus, Img2TzResult, Img2TzStatus, LoadCharResult, LoadCharStatus,
    MatchResult, MatchStatus, PasswordVerificationState, ReadSysParaResult, RegModelResult,
    RegModelStatus, Reply, SearchResult, SearchStatus, SystemParameters, TemplateNumResult,
    TemplateNumStatus, VfyPwdResult, StoreResult, StoreStatus, DeletCharResult, DeletCharStatus,
};
pub use crate::utils::Error;
