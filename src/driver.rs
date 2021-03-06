use arrayvec::ArrayVec;
use byteorder::{BigEndian, ByteOrder};
use core::cell::RefCell;
use embedded_hal::serial::{Read, Write};
use nb::block;

use crate::commands::Command;
use crate::responses::*;
use crate::utils::{CommandWriter, Error, FromPayload, ToPayload};

const REPLY_HEADER_LENGTH: u16 = 9;

/// Represents a R502 device connected to a U(S)ART.
///
/// A R502 has an address, which may mean that the intention is to use one USART line as a bus
/// network with multiple sensors attached to it. This is not explicitly supported by this driver.
#[derive(Debug)]
pub struct R502<TX, RX> {
    address: u32,
    tx: TX,
    rx: RX,
    received: ArrayVec<[u8; 1024]>,
    cmd_buffer: ArrayVec<[u8; 128]>,
    inflight_request: RefCell<Option<Command>>,
}

impl<TX, RX> CommandWriter for R502<TX, RX> {
    fn write_cmd_bytes(&mut self, bytes: &[u8]) {
        self.cmd_buffer.try_extend_from_slice(bytes).unwrap();
    }
}

impl<TX, RX> R502<TX, RX>
where
    TX: Write<u8>,
    RX: Read<u8>,
{
    /// Creates an instance of the R502. `tx` and `rx` are the transmit and receive halves of a
    /// USART, and `address` is the R502 address. By default this should be `0xffffffff`.
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

    /// Sends a command `cmd` to the R502 and then blocks waiting for the reply.
    /// The return value is either a response from the R502 or an error. Uses blocking USART
    /// API.
    ///
    /// # Errors
    ///
    /// ## `Error::WriteError(err)`
    /// Returned if the command could not be written to the serial port.
    /// Wraps the underlying error.
    ///
    /// ## `Error::ReadError(err)`
    /// Returned if the reply could not be read from the serial port.
    /// Wraps the underlying error.
    ///
    /// ## `Error::RecvPacketTooShort`
    /// Returned if the reply was only partially received.
    ///
    /// ## `Error::RecvWrongReplyType`
    /// Returned if the response packet was not a reply.
    pub fn send_command(&mut self, cmd: Command) -> Result<Reply, Error<TX::Error, RX::Error>> {
        self.cmd_buffer.clear();
        self.received.clear();
        self.prepare_cmd(cmd);

        let cmd_bytes = &self.cmd_buffer[..];
        for byte in cmd_bytes {
            match block!(self.tx.write(*byte)) {
                Err(e) => return Err(Error::WriteError(e)),
                Ok(..) => {}
            }
        }

        match block!(self.tx.flush()) {
            Err(e) => return Err(Error::WriteError(e)),
            Ok(..) => {}
        }

        return self.read_reply().and_then(|_| self.parse_reply());
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

    fn read_reply(&mut self) -> Result<u16, Error<TX::Error, RX::Error>> {
        // At first, we don't know the full packet size, so read in the
        // first 9 bytes of the packet header.
        for _ in 0..REPLY_HEADER_LENGTH {
            match block!(self.rx.read()) {
                Ok(word) => self.received.push(word),
                Err(error) => return Err(Error::RecvReadError(error)),
            }
        }

        let length = BigEndian::read_u16(&self.received[7..9]);
        for _ in 0..length {
            match block!(self.rx.read()) {
                Ok(word) => self.received.push(word),
                Err(error) => return Err(Error::RecvReadError(error)),
            }
        }

        return Ok(REPLY_HEADER_LENGTH + length);
    }

    fn parse_reply(&self) -> Result<Reply, Error<TX::Error, RX::Error>> {
        // Packet ID is in byte 6
        if self.received.len() < 7 {
            return Err(Error::RecvPacketTooShort);
        }

        // We have no business reading anything if there's no request in flight
        let inflight = self.inflight_request.borrow();
        if inflight.is_none() {
            return Err(Error::RecvUnsolicitedReply);
        }

        // We are looking for a response packet
        if self.received[6] != 0x07 {
            return Err(Error::RecvWrongReplyType);
        }

        return match *inflight {
            Some(Command::ReadSysPara) => Ok(Reply::ReadSysPara(ReadSysParaResult::from_payload(
                &self.received[..],
            ))),
            Some(Command::VfyPwd { .. }) => Ok(Reply::VfyPwd(VfyPwdResult::from_payload(
                &self.received[..],
            ))),
            Some(Command::GenImg) => Ok(Reply::GenImg(GenImgResult::from_payload(
                &self.received[..],
            ))),
            Some(Command::Img2Tz { .. }) => Ok(Reply::Img2Tz(Img2TzResult::from_payload(
                &self.received[..],
            ))),
            Some(Command::Search { .. }) => Ok(Reply::Search(SearchResult::from_payload(
                &self.received[..],
            ))),
            Some(Command::LoadChar { .. }) => Ok(Reply::LoadChar(LoadCharResult::from_payload(
                &self.received[..],
            ))),
            Some(Command::Match) => Ok(Reply::Match(MatchResult::from_payload(&self.received[..]))),
            Some(Command::TemplateNum) => Ok(Reply::TemplateNum(TemplateNumResult::from_payload(
                &self.received[..],
            ))),
            Some(Command::RegModel) => Ok(Reply::RegModel(RegModelResult::from_payload(
                &self.received[..],
            ))),
            Some(Command::Store { .. }) => Ok(Reply::Store(StoreResult::from_payload(
                &self.received[..],
            ))),
            Some(Command::DeletChar { .. }) => Ok(Reply::DeletChar(DeletCharResult::from_payload(
                &self.received[..],
            ))),
            None => panic!("Should not be reached"),
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
        assert_eq!(
            &r502.cmd_buffer[..],
            &[0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x03, 0x0f, 0x00, 0x13,]
        );
    }

    #[test]
    fn test_read_sys_para_deserialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();
        *r502.inflight_request.borrow_mut() = Some(Command::ReadSysPara);

        // and: a reply in the receive buffer
        r502.received
            .try_extend_from_slice(&[
                0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0x07, 0x00, 0x13, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0xc8, 0x00, 0x03, 0xff, 0xff, 0xff, 0xff, 0x00, 0x02, 0x00, 0x06, 0x04, 0xe9,
            ])
            .unwrap();

        // when: parsing a reply
        let r = r502.parse_reply();

        // then: reply is ok
        assert_eq!(r.is_ok(), true);

        // and: the reply is correct
        let reply = r.unwrap();
        match reply {
            Reply::ReadSysPara(ReadSysParaResult {
                address,
                confirmation_code: _,
                system_parameters,
                checksum: _,
            }) => {
                assert_eq!(address, 0xffffffff);
                assert_eq!(system_parameters.finger_library_size, 200);
            }
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
        r502.prepare_cmd(Command::VfyPwd {
            password: 0x00000000,
        });

        // then: the resulting packet length is ok
        assert_eq!(r502.cmd_buffer.len(), 16);
        // and: the packet is correct
        assert_eq!(
            &r502.cmd_buffer[..],
            &[
                0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x07, 0x13, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x1b,
            ]
        );
    }

    #[test]
    fn test_vfy_pwd_deserialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();
        *r502.inflight_request.borrow_mut() = Some(Command::VfyPwd {
            password: 0x00000000,
        });

        // and: a reply in the receive buffer
        r502.received
            .try_extend_from_slice(&[
                0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0x07, 0x00, 0x03, 0x00, 0x00, 0x0a,
            ])
            .unwrap();

        // when: parsing a reply
        let r = r502.parse_reply();

        // then: reply is ok
        assert_eq!(r.is_ok(), true);

        // and: the reply is correct
        let reply = r.unwrap();
        match reply {
            Reply::VfyPwd(VfyPwdResult {
                address,
                confirmation_code,
                checksum: _,
            }) => {
                assert_eq!(address, 0xffffffff);
                match confirmation_code {
                    PasswordVerificationState::Correct => (),
                    _ => panic!("Expected PasswordConfirmationCode::Correct"),
                };
            }
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
        assert_eq!(
            &r502.cmd_buffer[..],
            &[0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x03, 0x01, 0x00, 0x05,]
        );
    }

    #[test]
    fn test_gen_img_deserialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();
        *r502.inflight_request.borrow_mut() = Some(Command::GenImg);

        // and: a reply in the receive buffer
        r502.received
            .try_extend_from_slice(&[
                0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0x07, 0x00, 0x03, 0x00, 0x00, 0x0a,
            ])
            .unwrap();

        // when: parsing a reply
        let r = r502.parse_reply();

        // then: reply is ok
        assert_eq!(r.is_ok(), true);

        // and: the reply is correct
        let reply = r.unwrap();
        match reply {
            Reply::GenImg(GenImgResult {
                address,
                confirmation_code,
                checksum: _,
            }) => {
                assert_eq!(address, 0xffffffff);
                match confirmation_code {
                    GenImgStatus::Success => (),
                    _ => panic!("Expected GenImgStatus::Success"),
                };
            }
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
        assert_eq!(
            &r502.cmd_buffer[..],
            &[0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x04, 0x02, 0x01, 0x00, 0x08,]
        );
    }

    #[test]
    fn test_img_2_tz_deserialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();
        *r502.inflight_request.borrow_mut() = Some(Command::Img2Tz { buffer: 1 });

        // and: a reply in the receive buffer
        r502.received
            .try_extend_from_slice(&[
                0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0x07, 0x00, 0x03, 0x00, 0x00, 0x0a,
            ])
            .unwrap();

        // when: parsing a reply
        let r = r502.parse_reply();

        // then: reply is ok
        assert_eq!(r.is_ok(), true);

        // and: the reply is correct
        let reply = r.unwrap();
        match reply {
            Reply::Img2Tz(Img2TzResult {
                address,
                confirmation_code,
                checksum: _,
            }) => {
                assert_eq!(address, 0xffffffff);
                match confirmation_code {
                    Img2TzStatus::Success => (),
                    _ => panic!("Expected Img2TzStatus::Success"),
                };
            }
            _ => panic!("Expected Reply::Img2Tz, got something else!"),
        };
    }

    #[test]
    fn test_search_serialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();

        // when: preparing a GenImg command
        r502.prepare_cmd(Command::Search {
            buffer: 1,
            start_index: 0,
            end_index: 0xffff,
        });

        // then: the resulting packet length is correct
        assert_eq!(r502.cmd_buffer.len(), 17);
        // and: the packet is correct
        assert_eq!(
            &r502.cmd_buffer[..],
            &[
                0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x08, 0x04, 0x01, 0x00, 0x00, 0xff,
                0xff, 0x02, 0x0c,
            ]
        );
    }

    #[test]
    fn test_search_deserialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();
        *r502.inflight_request.borrow_mut() = Some(Command::Search {
            buffer: 1,
            start_index: 0,
            end_index: 0xffff,
        });

        // and: a reply in the receive buffer
        r502.received
            .try_extend_from_slice(&[
                0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0x07, 0x00, 0x07, 0x00, 0x00, 0x00, 0x00, 0xff,
                0x00, 0x4a,
            ])
            .unwrap();

        // when: parsing a reply
        let r = r502.parse_reply();

        // then: reply is ok
        assert_eq!(r.is_ok(), true);

        // and: the reply is correct
        let reply = r.unwrap();
        match reply {
            Reply::Search(SearchResult {
                address,
                confirmation_code,
                match_id,
                match_score,
                checksum: _,
            }) => {
                assert_eq!(address, 0xffffffff);
                match confirmation_code {
                    SearchStatus::Success => (),
                    _ => panic!("Expected SearchStatus::Success"),
                };
                assert_eq!(match_id, 0);
                assert_eq!(match_score, 255);
            }
            _ => panic!("Expected Reply::Search, got something else!"),
        };
    }

    #[test]
    fn test_load_char_serialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();

        // when: preparing a GenImg command
        r502.prepare_cmd(Command::LoadChar {
            buffer: 2,
            index: 0,
        });

        // then: the resulting packet length is correct
        assert_eq!(r502.cmd_buffer.len(), 15);
        // and: the packet is correct
        assert_eq!(
            &r502.cmd_buffer[..],
            &[
                0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x06, 0x07, 0x02, 0x00, 0x00, 0x00,
                0x10,
            ]
        );
    }

    #[test]
    fn test_load_char_tz_deserialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();
        *r502.inflight_request.borrow_mut() = Some(Command::LoadChar {
            buffer: 2,
            index: 0,
        });

        // and: a reply in the receive buffer
        r502.received
            .try_extend_from_slice(&[
                0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0x07, 0x00, 0x03, 0x00, 0x00, 0x0a,
            ])
            .unwrap();

        // when: parsing a reply
        let r = r502.parse_reply();

        // then: reply is ok
        assert_eq!(r.is_ok(), true);

        // and: the reply is correct
        let reply = r.unwrap();
        match reply {
            Reply::LoadChar(LoadCharResult {
                address,
                confirmation_code,
                checksum: _,
            }) => {
                assert_eq!(address, 0xffffffff);
                match confirmation_code {
                    LoadCharStatus::Success => (),
                    _ => panic!("Expected LoadCharStatus::Success"),
                };
            }
            _ => panic!("Expected Reply::LoadChar, got something else!"),
        };
    }

    #[test]
    fn test_match_serialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();

        // when: preparing a GenImg command
        r502.prepare_cmd(Command::Match);

        // then: the resulting packet length is correct
        assert_eq!(r502.cmd_buffer.len(), 12);
        // and: the packet is correct
        assert_eq!(
            &r502.cmd_buffer[..],
            &[0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x03, 0x03, 0x00, 0x07,]
        );
    }

    #[test]
    fn test_match_deserialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();
        *r502.inflight_request.borrow_mut() = Some(Command::Match);

        // and: a reply in the receive buffer
        r502.received
            .try_extend_from_slice(&[
                0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0x07, 0x00, 0x05, 0x00, 0x00, 0x32, 0x00, 0x3e,
            ])
            .unwrap();

        // when: parsing a reply
        let r = r502.parse_reply();

        // then: reply is ok
        assert_eq!(r.is_ok(), true);

        // and: the reply is correct
        let reply = r.unwrap();
        match reply {
            Reply::Match(MatchResult {
                address,
                confirmation_code,
                match_score,
                checksum: _,
            }) => {
                assert_eq!(address, 0xffffffff);
                match confirmation_code {
                    MatchStatus::Success => (),
                    _ => panic!("Expected MatchStatus::Success"),
                };
                assert_eq!(match_score, 50);
            }
            _ => panic!("Expected Reply::Match, got something else!"),
        };
    }

    #[test]
    fn test_template_num_serialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();

        // when: preparing a GenImg command
        r502.prepare_cmd(Command::TemplateNum);

        // then: the resulting packet length is correct
        assert_eq!(r502.cmd_buffer.len(), 12);
        // and: the packet is correct
        assert_eq!(
            &r502.cmd_buffer[..],
            &[0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x03, 0x1d, 0x00, 0x21,]
        );
    }

    #[test]
    fn test_template_num_deserialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();
        *r502.inflight_request.borrow_mut() = Some(Command::TemplateNum);

        // and: a reply in the receive buffer
        r502.received
            .try_extend_from_slice(&[
                0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0x07, 0x00, 0x05, 0x00, 0x00, 0x03, 0x00, 0x0f,
            ])
            .unwrap();

        // when: parsing a reply
        let r = r502.parse_reply();

        // then: reply is ok
        assert_eq!(r.is_ok(), true);

        // and: the reply is correct
        let reply = r.unwrap();
        match reply {
            Reply::TemplateNum(TemplateNumResult {
                address,
                confirmation_code,
                template_num,
                checksum: _,
            }) => {
                assert_eq!(address, 0xffffffff);
                match confirmation_code {
                    TemplateNumStatus::Success => (),
                    _ => panic!("Expected TemplateNumStatus::Success"),
                };
                assert_eq!(template_num, 3);
            }
            _ => panic!("Expected Reply::TemplateNum, got something else!"),
        };
    }

    #[test]
    fn test_reg_model_serialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();

        // when: preparing a GenImg command
        r502.prepare_cmd(Command::RegModel);

        // then: the resulting packet length is correct
        assert_eq!(r502.cmd_buffer.len(), 12);
        // and: the packet is correct
        assert_eq!(
            &r502.cmd_buffer[..],
            &[0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x03, 0x05, 0x00, 0x09,]
        );
    }

    #[test]
    fn test_reg_model_deserialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();
        *r502.inflight_request.borrow_mut() = Some(Command::RegModel);

        // and: a reply in the receive buffer
        r502.received
            .try_extend_from_slice(&[
                0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0x07, 0x00, 0x03, 0x00, 0x00, 0x0a,
            ])
            .unwrap();

        // when: parsing a reply
        let r = r502.parse_reply();

        // then: reply is ok
        assert_eq!(r.is_ok(), true);

        // and: the reply is correct
        let reply = r.unwrap();
        match reply {
            Reply::RegModel(RegModelResult {
                address,
                confirmation_code,
                checksum: _,
            }) => {
                assert_eq!(address, 0xffffffff);
                match confirmation_code {
                    RegModelStatus::Success => (),
                    _ => panic!("Expected RegModelStatus::Success"),
                };
            }
            _ => panic!("Expected Reply::RegModel, got something else!"),
        };
    }

    #[test]
    fn test_store_serialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();

        // when: preparing a GenImg command
        r502.prepare_cmd(Command::Store { buffer: 1, index: 4 });

        // then: the resulting packet length is correct
        assert_eq!(r502.cmd_buffer.len(), 15);
        // and: the packet is correct
        assert_eq!(
            &r502.cmd_buffer[..],
            &[
                0xef,
                0x01,
                0xff,
                0xff,
                0xff,
                0xff,
                0x01,
                0x00,
                0x06,
                0x06,
                0x01,
                0x00,
                0x04,
                0x00,
                0x12,
            ]
        );
    }

    #[test]
    fn test_store_deserialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();
        *r502.inflight_request.borrow_mut() = Some(Command::Store { index: 1, buffer: 1});

        // and: a reply in the receive buffer
        r502.received
            .try_extend_from_slice(&[
                0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0x07, 0x00, 0x03, 0x00, 0x00, 0x0a,
            ])
            .unwrap();

        // when: parsing a reply
        let r = r502.parse_reply();

        // then: reply is ok
        assert_eq!(r.is_ok(), true);

        // and: the reply is correct
        let reply = r.unwrap();
        match reply {
            Reply::Store(StoreResult {
                address,
                confirmation_code,
                checksum: _,
            }) => {
                assert_eq!(address, 0xffffffff);
                match confirmation_code {
                    StoreStatus::Success => (),
                    _ => panic!("Expected StoreStatus::Success"),
                };
            }
            _ => panic!("Expected Reply::Store, got something else!"),
        };
    }

    #[test]
    fn test_delet_char_serialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();

        // when: preparing a GenImg command
        r502.prepare_cmd(Command::DeletChar { start_index: 4, num_to_delete: 1 });

        // then: the resulting packet length is correct
        assert_eq!(r502.cmd_buffer.len(), 16);
        // and: the packet is correct
        assert_eq!(
            &r502.cmd_buffer[..],
            &[
                0xef,
                0x01,
                0xff,
                0xff,
                0xff,
                0xff,
                0x01,
                0x00,
                0x07,
                0x0c,
                0x00,
                0x04,
                0x00,
                0x01,
                0x00,
                0x19,
            ]
        );
    }

    #[test]
    fn test_delet_char_deserialisation() {
        // given: a r502 instance
        let mut r502 = R502::new(TestTx, TestRx, 0xffffffff);
        r502.cmd_buffer.clear();
        r502.received.clear();
        *r502.inflight_request.borrow_mut() = Some(Command::DeletChar { start_index: 1, num_to_delete: 1});

        // and: a reply in the receive buffer
        r502.received
            .try_extend_from_slice(&[
                0xef, 0x01, 0xff, 0xff, 0xff, 0xff, 0x07, 0x00, 0x03, 0x00, 0x00, 0x0a,
            ])
            .unwrap();

        // when: parsing a reply
        let r = r502.parse_reply();

        // then: reply is ok
        assert_eq!(r.is_ok(), true);

        // and: the reply is correct
        let reply = r.unwrap();
        match reply {
            Reply::DeletChar(DeletCharResult {
                address,
                confirmation_code,
                checksum: _,
            }) => {
                assert_eq!(address, 0xffffffff);
                match confirmation_code {
                    DeletCharStatus::Success => (),
                    _ => panic!("Expected DeletCharStatus::Success"),
                };
            }
            _ => panic!("Expected Reply::DeletChar, got something else!"),
        };
    }
}
