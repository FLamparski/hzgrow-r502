use crate::utils::{ToPayload, CommandWriter};
//# Naming conventions etc follow the R502 datasheet, see:
//# https://www.dropbox.com/sh/epucei8lmoz7xpp/AAAmon04b1DiSOeh1q4nAhzAa?dl=0&preview=R502+fingerprint+module+user+manual-V1.2.pdf

/// Enum for commands one can send to the R502. Names match the datasheet.
#[derive(Debug)]
pub enum Command {
    /// Reads system status and basic configuration
    ReadSysPara,

    /// Performs a handshake with the device to verify the password.
    /// The default password on the R502 is 0x00000000.
    VfyPwd {
        /// The device password.
        password: u32,
    },

    /// Captures an image of the fingerprint
    GenImg,

    /// Processes an image into a _character buffer_
    Img2Tz {
        /// Which buffer to store the processed fingerprint data into (there are 2).
        /// 
        /// **Note:** The buffers are named **1** and **2**. Any other value defaults to 2.
        buffer: u8,
    },
}

impl ToPayload
for Command {
    fn to_payload(&self, writer: &mut dyn CommandWriter) {
        match self {
            // Required packet:
            // headr  | 0xEF 0x01 [2]
            // addr   | cmd.address [4]
            // ident  | 0x01 [1]
            // length | 0x00 0x03 [2]
            // instr  | 0x0F [1]
            // chksum | checksum [2]
            Self::ReadSysPara => {
                writer.write_cmd_bytes(&[0x01]);
                writer.write_cmd_bytes(&[0x00, 0x03]);
                writer.write_cmd_bytes(&[0x0F]);
            },

            // Required packet:
            // headr  | 0xEF 0x01 [2]
            // addr   | cmd.address [4]
            // ident  | 0x01 [1]
            // length | 0x00 0x07 [2]
            // instr  | 0x13 [1]
            // passwd | cmd.password [4]
            // chksum | checksum [2]
            Self::VfyPwd { password } => {
                writer.write_cmd_bytes(&[0x01]);
                writer.write_cmd_bytes(&[0x00, 0x07]);
                writer.write_cmd_bytes(&[0x13]);
                writer.write_cmd_bytes(&password.to_be_bytes()[..]);
            },

            // Required packet:
            // headr  | 0xEF 0x01 [2]
            // addr   | cmd.address [4]
            // ident  | 0x01 [1]
            // length | 0x00 0x01 [2]
            // instr  | 0x0F [1]
            // chksum | checksum [2]
            Self::GenImg => {
                writer.write_cmd_bytes(&[0x01]);
                writer.write_cmd_bytes(&[0x00, 0x03]);
                writer.write_cmd_bytes(&[0x01]);
            }

            Self::Img2Tz { buffer } => {
                writer.write_cmd_bytes(&[0x01]);
                writer.write_cmd_bytes(&[0x00, 0x04]);
                writer.write_cmd_bytes(&[0x02]);
                writer.write_cmd_bytes(&[*buffer]);
            }
        }
    }
}
