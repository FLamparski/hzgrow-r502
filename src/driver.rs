use embedded_hal::{serial::{Read, Write}};
use arrayvec::ArrayVec;
use nb::block;
use byteorder::{ByteOrder, BigEndian};
use core::cell::RefCell;

use crate::commands::Command;
use crate::responses::*;
use crate::utils::{FromPayload, CommandWriter, ToPayload};

const REPLY_HEADER_LENGTH: u16 = 9;

/// Represents a R502 device connected to a U(S)ART.
#[derive(Debug)]
pub struct R502<TX, RX> {
    address: u32,
    tx: TX,
    rx: RX,
    received: ArrayVec<[u8; 1024]>,
    cmd_buffer: ArrayVec<[u8; 128]>,
    inflight_request: RefCell<Option<Command>>,
}

impl<TX, RX> CommandWriter
for R502<TX, RX> {
    fn write_cmd_bytes(&mut self, bytes: &[u8]) {
        self.cmd_buffer.try_extend_from_slice(bytes).unwrap();
    }
}

impl<TX, RX> R502<TX, RX>
where TX: Write<u8>,
      RX: Read<u8>
{
    pub fn new(tx: TX, rx: RX, address: u32) -> Self {
        Self {
            address: address,
            tx: tx,
            rx: rx,
            received: ArrayVec::<[u8; 1024]>::new(),
            cmd_buffer: ArrayVec::<[u8; 128]>::new(),
            inflight_request: RefCell::from(None),
        }
    }

    /// Sends a command to the R502 and then blocks waiting for the reply.
    /// The return value is either a response from the R502 or an `Err(())`.
    /// 
    /// TODO: Add better error results.
    pub fn send_command(&mut self, cmd: Command) -> Result<Reply, ()> {
        self.cmd_buffer.clear();
        self.received.clear();
        self.prepare_cmd(cmd);

        let cmd_bytes = &self.cmd_buffer[..];
        for byte in cmd_bytes {
            block!(self.tx.write(*byte)).ok();
        }

        block!(self.tx.flush()).ok();

        if self.read_reply().is_some() {
            if let Some(reply) = self.parse_reply() {
                return Result::Ok(reply);
            }
        }

        return Result::Err(());
    }

    fn prepare_cmd(&mut self, cmd: Command) {
        self.write_header(self.address);
        cmd.to_payload(self);
        let chk = self.compute_checksum();
        self.write_cmd_bytes(&chk.to_be_bytes()[..]);

        *self.inflight_request.borrow_mut() = Some(cmd);
    }

    fn write_header(&mut self, address: u32) {
        self.write_cmd_bytes(&[0xEF, 0x01]);
        self.write_cmd_bytes(&address.to_be_bytes()[..]);
    }

    fn compute_checksum(&self) -> u16 {
        let mut checksum = 0u16;
        let check_end = self.cmd_buffer.len();
        let checked_bytes = &self.cmd_buffer[6..check_end];
        for byte in checked_bytes {
            checksum += (*byte) as u16;
        }
        return checksum;
    }

    fn read_reply(&mut self) -> Option<u16> {
        // At first, we don't know the full packet size, so read in the
        // first 9 bytes of the packet header.
        for _ in 0..REPLY_HEADER_LENGTH {
            if let Some(byte) = block!(self.rx.read()).ok() {
                self.received.push(byte);
            } else {
                return None;
            }
        }

        let length = BigEndian::read_u16(&self.received[7..9]);
        for _ in 0..length {
            if let Some(byte) = block!(self.rx.read()).ok() {
                self.received.push(byte);
            } else {
                return None;
            }
        }

        return Some(REPLY_HEADER_LENGTH + length);
    }

    fn parse_reply(&self) -> Option<Reply> {
        // Packet ID is in byte 6
        if self.received.len() < 7 {
            return None;
        }

        // We have no business reading anything if there's no request in flight
        let inflight = self.inflight_request.borrow();
        if inflight.is_none() {
            return None;
        }

        // We are looking for a response packet
        if self.received[6] != 0x07 {
            return None;
        }

        return match *inflight {
            Some(Command::ReadSysPara) => {
                Some(Reply::ReadSysPara(ReadSysParaResult::from_payload(&self.received[..])))
            },
            Some(Command::VfyPwd { password: _ }) => {
                Some(Reply::VfyPwd(VfyPwdResult::from_payload(&self.received[..])))
            },
            Some(Command::GenImg) => {
                Some(Reply::GenImg(GenImgResult::from_payload(&self.received[..])))
            },
            Some(Command::Img2Tz { buffer: _ }) => {
                Some(Reply::Img2Tz(Img2TzResult::from_payload(&self.received[..])))
            },
            Some(Command::Search { buffer: _, start_index: _, end_index: _ }) => {
                Some(Reply::Search(SearchResult::from_payload(&self.received[..])))
            },
            None => None
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTx;
    struct TestRx;
    
    impl Write<u8> for TestTx {
        type Error = ();
        fn write(&mut self, _word: u8) -> nb::Result<(), Self::Error> {
            return Ok(());
        }
        fn flush(&mut self) -> nb::Result<(), Self::Error> {
            return Ok(());
        }
    }

    impl Read<u8> for TestRx {
        type Error = ();
        fn read(&mut self) -> nb::Result<u8, Self::Error> {
            return Ok(0u8);
        }
    }
    
    #[test]
    fn checksum_tests() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();

        // and: some data to compute a checksum of
        r502.write_cmd_bytes(&[0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0xc0, 0xc1]);

        // when: computing the command checksum
        // then: the checksum is correct
        assert_eq!(r502.compute_checksum(), 0x0181u16);
    }

    #[test]
    fn test_read_sys_para_serialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();

        // when: preparing a ReadSysPara command
        r502.prepare_cmd(Command::ReadSysPara);

        // then: the resulting packet length is correct
        assert_eq!(r502.cmd_buffer.len(), 12);
        // and: the packet is correct
        assert_eq!(&r502.cmd_buffer[..], &[
            0xef,
            0x01,
            0xff,
            0xff,
            0xff,
            0xff,
            0x01,
            0x00,
            0x03,
            0x0f,
            0x00,
            0x13,
        ]);
    }

    #[test]
    fn test_read_sys_para_deserialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();
        *r502.inflight_request.borrow_mut() = Some(Command::ReadSysPara);

        // and: a reply in the receive buffer
        r502.received.try_extend_from_slice(&[
            0xef,
            0x01,
            0xff,
            0xff,
            0xff,
            0xff,
            0x07,
            0x00,
            0x13,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0xc8,
            0x00,
            0x03,
            0xff,
            0xff,
            0xff,
            0xff,
            0x00,
            0x02,
            0x00,
            0x06,
            0x04,
            0xe9,
        ]).unwrap();

        // when: parsing a reply
        let r = r502.parse_reply();

        // then: reply is ok
        assert_eq!(r.is_some(), true);

        // and: the reply is correct
        let reply = r.unwrap();
        match reply {
            Reply::ReadSysPara(ReadSysParaResult { address, confirmation_code: _, system_parameters, checksum: _ }) => {
                assert_eq!(address, 0xffffffff);
                assert_eq!(system_parameters.finger_library_size, 200);
            },
            _ => panic!("Expected Reply::ReadSysPara, got something else!"),
        };
    }

    #[test]
    fn vfy_pwd_serialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();

        // when: preparing a VfyPwd command
        r502.prepare_cmd(Command::VfyPwd { password: 0x00000000 });

        // then: the resulting packet length is ok
        assert_eq!(r502.cmd_buffer.len(), 16);
        // and: the packet is correct
        assert_eq!(&r502.cmd_buffer[..], &[
            0xef,
            0x01,
            0xff,
            0xff,
            0xff,
            0xff,
            0x01,
            0x00,
            0x07,
            0x13,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x1b,
        ]);
    }

    #[test]
    fn test_vfy_pwd_deserialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();
        *r502.inflight_request.borrow_mut() = Some(Command::VfyPwd { password: 0x00000000 });

        // and: a reply in the receive buffer
        r502.received.try_extend_from_slice(&[
            0xef,
            0x01,
            0xff,
            0xff,
            0xff,
            0xff,
            0x07,
            0x00,
            0x03,
            0x00,
            0x00,
            0x0a,
        ]).unwrap();

        // when: parsing a reply
        let r = r502.parse_reply();

        // then: reply is ok
        assert_eq!(r.is_some(), true);

        // and: the reply is correct
        let reply = r.unwrap();
        match reply {
            Reply::VfyPwd(VfyPwdResult { address, confirmation_code, checksum: _ }) => {
                assert_eq!(address, 0xffffffff);
                match confirmation_code {
                    PasswordVerificationState::Correct => (),
                    _ => panic!("Expected PasswordConfirmationCode::Correct"),
                };
            },
            _ => panic!("Expected Reply::VfyPwd, got something else!"),
        };
    }

    #[test]
    fn gen_img_serialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();
        
        // when: preparing a GenImg command
        r502.prepare_cmd(Command::GenImg);

        // then: the resulting packet length is correct
        assert_eq!(r502.cmd_buffer.len(), 12);
        // and: the packet is correct
        assert_eq!(&r502.cmd_buffer[..], &[
            0xef,
            0x01,
            0xff,
            0xff,
            0xff,
            0xff,
            0x01,
            0x00,
            0x03,
            0x01,
            0x00,
            0x05,
        ]);
    }

    #[test]
    fn test_gen_img_deserialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();
        *r502.inflight_request.borrow_mut() = Some(Command::GenImg);

        // and: a reply in the receive buffer
        r502.received.try_extend_from_slice(&[
            0xef,
            0x01,
            0xff,
            0xff,
            0xff,
            0xff,
            0x07,
            0x00,
            0x03,
            0x00,
            0x00,
            0x0a,
        ]).unwrap();

        // when: parsing a reply
        let r = r502.parse_reply();

        // then: reply is ok
        assert_eq!(r.is_some(), true);

        // and: the reply is correct
        let reply = r.unwrap();
        match reply {
            Reply::GenImg(GenImgResult { address, confirmation_code, checksum: _ }) => {
                assert_eq!(address, 0xffffffff);
                match confirmation_code {
                    GenImgStatus::Success => (),
                    _ => panic!("Expected GenImgStatus::Success"),
                };
            },
            _ => panic!("Expected Reply::GenImg, got something else!"),
        };
    }

    #[test]
    fn test_img_2_tz_serialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();
        
        // when: preparing a GenImg command
        r502.prepare_cmd(Command::Img2Tz { buffer: 1 });

        // then: the resulting packet length is correct
        assert_eq!(r502.cmd_buffer.len(), 13);
        // and: the packet is correct
        assert_eq!(&r502.cmd_buffer[..], &[
            0xef,
            0x01,
            0xff,
            0xff,
            0xff,
            0xff,
            0x01,
            0x00,
            0x04,
            0x02,
            0x01,
            0x00,
            0x08,
        ]);
    }

    #[test]
    fn test_img_2_tz_deserialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();
        *r502.inflight_request.borrow_mut() = Some(Command::Img2Tz { buffer: 1 });

        // and: a reply in the receive buffer
        r502.received.try_extend_from_slice(&[
            0xef,
            0x01,
            0xff,
            0xff,
            0xff,
            0xff,
            0x07,
            0x00,
            0x03,
            0x00,
            0x00,
            0x0a,
        ]).unwrap();

        // when: parsing a reply
        let r = r502.parse_reply();

        // then: reply is ok
        assert_eq!(r.is_some(), true);

        // and: the reply is correct
        let reply = r.unwrap();
        match reply {
            Reply::Img2Tz(Img2TzResult { address, confirmation_code, checksum: _ }) => {
                assert_eq!(address, 0xffffffff);
                match confirmation_code {
                    Img2TzStatus::Success => (),
                    _ => panic!("Expected Img2TzStatus::Success"),
                };
            },
            _ => panic!("Expected Reply::Img2Tz, got something else!"),
        };
    }
}
