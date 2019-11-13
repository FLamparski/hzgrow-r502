use embedded_hal::{serial::{Read, Write}};
use arrayvec::ArrayVec;
use nb::block;
use byteorder::{ByteOrder, BigEndian};

use crate::commands::{Command, Reply, SystemParameters};

/// Represents a R502 device connected to a U(S)ART.
#[derive(Debug)]
pub struct R502<TX, RX> {
    tx: TX,
    rx: RX,
    received: ArrayVec<[u8; 1024]>,
    cmd_buffer: ArrayVec<[u8; 128]>,
}

impl<TX, RX> R502<TX, RX>
where TX: Write<u8>,
      RX: Read<u8>
{
    pub fn new(tx: TX, rx: RX) -> Self {
        Self {
            tx: tx,
            rx: rx,
            received: ArrayVec::<[u8; 1024]>::new(),
            cmd_buffer: ArrayVec::<[u8; 128]>::new(),
        }
    }

    /// Sends a command to the R502 and then blocks waiting for the reply.
    /// The return value is either a response from the R502 or an `Err(())`.
    /// 
    /// TODO: Add better error results.
    pub fn send_command(&mut self, cmd: Command) -> Result<Reply, ()> {
        self.cmd_buffer.clear();
        let response_len = self.prepare_cmd(cmd);

        let cmd_bytes = &self.cmd_buffer[..];
        for byte in cmd_bytes {
            block!(self.tx.write(*byte)).ok();
        }

        for i in 0..response_len {
            if let Some(byte) = block!(self.rx.read()).ok() {
                self.received[i] = byte;
            } else {
                return Result::Err(());
            }
        }

        if let Some(reply) = self.parse_reply() {
            return Result::Ok(reply);
        }
        return Result::Err(());
    }

    fn prepare_cmd(&mut self, cmd: Command) -> usize {
        match cmd {
            Command::ReadSysPara { address } => {
                // Required packet:
                // headr  | 0xEF 0x01 [2]
                // addr   | cmd.address [4]
                // ident  | 0x01 [1]
                // length | 0x00 0x03 [2]
                // instr  | 0x0F [1]
                // chksum | checksum [2]
                self.write_cmd_bytes(&[0xEF, 0x01]);
                self.write_cmd_bytes(&address.to_be_bytes()[..]);
                self.write_cmd_bytes(&[0x01]);
                self.write_cmd_bytes(&[0x00, 0x03]);
                self.write_cmd_bytes(&[0x0F]);
                let chk = self.compute_checksum();
                self.write_cmd_bytes(&chk.to_be_bytes()[..]);
                return 28;
            },
        };
    }

    fn write_cmd_bytes(&mut self, bytes: &[u8]) {
        self.cmd_buffer.try_extend_from_slice(bytes).unwrap();
    }

    fn compute_checksum(&self) -> u16 {
        let mut checksum = 0u16;
        let check_end = self.cmd_buffer.len() - 2;
        let checked_bytes = &self.cmd_buffer[6..check_end];
        for byte in checked_bytes {
            checksum += (*byte) as u16;
        }
        return checksum;
    }

    fn parse_reply(&self) -> Option<Reply> {
        // Packet ID is in byte 6
        if self.received.len() < 7 {
            return None;
        }

        return match self.received[6] {
            0x07 => {
                // Expected packet:
                // headr  | 0xEF 0x01 [2]
                // addr   | cmd.address [4]
                // ident  | 0x01 [1]
                // length | 0x00 0x03 [2] == 19 (3 + 16)
                // confrm | 0x0F [1]
                // params | (params) [16]
                // chksum | checksum [2]
                Some(Reply::ReadSysPara {
                    address: BigEndian::read_u32(&self.received[2..6]),
                    confirmation_code: self.received[9],
                    checksum: BigEndian::read_u16(&self.received[25..27]),
                    system_parameters: SystemParameters::from_payload(&self.received[9..25])
                })
            },
            _ => None,
        };
    }
}

trait FromPayload<T> {
    fn from_payload(payload: &[u8]) -> T;
}

impl FromPayload<SystemParameters>
for SystemParameters {
    fn from_payload(payload: &[u8]) -> SystemParameters {
        // HZ R502's datasheet is a little inconsistent - sometimes the sizes are given in bytes
        // and sometimes in words; words are 16 bit (2 byte).
        // Pick a flipping unit and stick with it!
        SystemParameters {
            status_register: BigEndian::read_u16(&payload[0..2]),
            system_identifier_code: BigEndian::read_u16(&payload[2..4]),
            finger_library_size: BigEndian::read_u16(&payload[4..6]),
            security_level: BigEndian::read_u16(&payload[6..8]),
            device_address: BigEndian::read_u32(&payload[8..12]),
            packet_size: BigEndian::read_u16(&payload[12..14]),
            baud_setting: BigEndian::read_u16(&payload[12..16]),
        }
    }
}
