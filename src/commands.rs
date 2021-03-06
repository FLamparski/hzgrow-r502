use crate::utils::{CommandWriter, ToPayload};

/// Commands that one can send to the R502.
///
/// Command naming and some field names are taken from the R502 datasheet: [Datasheet link](https://www.dropbox.com/sh/epucei8lmoz7xpp/AAAmon04b1DiSOeh1q4nAhzAa?dl=0&preview=R502+fingerprint+module+user+manual-V1.2.pdf) -
/// yes, it actually is hosted on Dropbox.
#[derive(Debug)]
pub enum Command {
    /// Reads system status and configuration
    ReadSysPara,

    /// Performs a handshake with the device to verify the password.
    /// The default password on the R502 is 0x00000000.
    VfyPwd {
        /// The device password.
        password: u32,
    },

    /// Captures an image of the fingerprint into the _image buffer_.
    GenImg,

    /// Processes the image from the R502's _image buffer_ into one of the two
    /// available _character buffers_. This command actually runs the image recognition
    /// and builds a feature vector-like representation of the fingerprint captured.
    Img2Tz {
        /// Which buffer to store the processed fingerprint data into (there are 2).
        ///
        /// **Note:** The buffers are named **1** and **2**. Any other value defaults to 2.
        buffer: u8,
    },

    /// Matches the captured fingerprint against a number of stored templates. You can set the
    /// `start_index` and `end_index` to `0` and `0xff` respectively to search the entire library.
    Search {
        /// Which buffer to store the processed fingerprint data into (there are 2).
        ///
        /// **Note:** The buffers are named **1** and **2**. Any other value defaults to 2.
        buffer: u8,

        /// The start index. Where the search should start from. 0-based.
        start_index: u16,

        /// The end index. Where the search should stop. No word on whether this is inclusive or
        /// exclusive.
        end_index: u16,
    },

    /// Loads a fingerprint _character file_ into one of the two _character buffers_.
    LoadChar {
        /// Which buffer to store the processed fingerprint data into (there are 2).
        ///
        /// **Note:** The buffers are named **1** and **2**. Any other value defaults to 2.
        buffer: u8,

        /// Which fingerprint to load from the library (0-based index).
        index: u16,
    },

    /// Performs a match between the two _character buffers_. This will typically be used
    /// to match a new fingerprint against a known template, to verify that the correct
    /// finger is placed on the reader.
    Match,

    /// Returns the next valid index at which a new fingerprint can be enrolled.
    TemplateNum,

    /// Combines fingerprint data stored in two _character buffers_ into a new _template_,
    /// which is returned into _both_ character buffers. This is part of the enrollment
    /// process. For this to work, both buffers need to contain data from the same finger.
    RegModel,

    /// Stores a fingerprint template from the given buffer into the library.
    /// 
    /// **Note:** This will allow you to overwrite an existing fingerprint template.
    /// Use with caution, and invoke `TemplateNum` first to get the next free index.
    Store {
        /// Which _character buffer_ to read the fingerprint template from (there are 2).
        ///
        /// **Note:** The buffers are named **1** and **2**. Any other value defaults to 2.
        /// Also note that it shouldn't really matter which buffer you use when enrolling, since
        /// `RegModel` will return its result into both buffers.
        buffer: u8,

        /// Which index in the library to store the fingerprint template into.
        index: u16,
    },

    /// Deletes enrolled fingerprint templates starting from the given index.
    DeletChar {
        /// Index of the fingerprint template in the library to delete.
        start_index: u16,

        /// Number of templates to delete starting from the given index
        num_to_delete: u16,
    },
}

impl ToPayload for Command {
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
            }

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
            }

            // Required packet:
            // headr  | 0xEF 0x01 [2]
            // addr   | cmd.address [4]
            // ident  | 0x01 [1]
            // length | 0x00 0x03 [2]
            // instr  | 0x01 [1]
            // chksum | checksum [2]
            Self::GenImg => {
                writer.write_cmd_bytes(&[0x01]);
                writer.write_cmd_bytes(&[0x00, 0x03]);
                writer.write_cmd_bytes(&[0x01]);
            }

            // Required packet:
            // headr  | 0xEF 0x01 [2]
            // addr   | cmd.address [4]
            // ident  | 0x01 [1]
            // length | 0x00 0x04 [2]
            // instr  | 0x02 [1]
            // bufid  | buffer [1]
            // chksum | checksum [2]
            Self::Img2Tz { buffer } => {
                writer.write_cmd_bytes(&[0x01]);
                writer.write_cmd_bytes(&[0x00, 0x04]);
                writer.write_cmd_bytes(&[0x02]);
                writer.write_cmd_bytes(&[*buffer]);
            }

            // Required packet:
            // headr  | 0xEF 0x01 [2]
            // addr   | cmd.address [4]
            // ident  | 0x01 [1]
            // length | 0x00 0x08 [2]
            // instr  | 0x04 [1]
            // bufid  | buffer [1]
            // sstart | start_index [2]
            // send   | end_index [2]
            // chksum | checksum [2]
            Self::Search {
                buffer,
                start_index,
                end_index,
            } => {
                writer.write_cmd_bytes(&[0x01]);
                writer.write_cmd_bytes(&[0x00, 0x08]);
                writer.write_cmd_bytes(&[0x04]);
                writer.write_cmd_bytes(&[*buffer]);
                writer.write_cmd_bytes(&start_index.to_be_bytes()[..]);
                writer.write_cmd_bytes(&end_index.to_be_bytes()[..]);
            }

            // Required packet:
            // headr  | 0xEF 0x01 [2]
            // addr   | cmd.address [4]
            // ident  | 0x01 [1]
            // length | 0x00 0x06 [2]
            // instr  | 0x07 [1]
            // bufid  | buffer [1]
            // sstart | index [2]
            // chksum | checksum [2]
            Self::LoadChar { buffer, index } => {
                writer.write_cmd_bytes(&[0x01]);
                writer.write_cmd_bytes(&[0x00, 0x06]);
                writer.write_cmd_bytes(&[0x07]);
                writer.write_cmd_bytes(&[*buffer]);
                writer.write_cmd_bytes(&index.to_be_bytes()[..]);
            }

            // Required packet:
            // headr  | 0xEF 0x01 [2]
            // addr   | cmd.address [4]
            // ident  | 0x01 [1]
            // length | 0x00 0x03 [2]
            // instr  | 0x03 [1]
            // chksum | checksum [2]
            Self::Match => {
                writer.write_cmd_bytes(&[0x01]);
                writer.write_cmd_bytes(&[0x00, 0x03]);
                writer.write_cmd_bytes(&[0x03]);
            }

            // Required packet:
            // headr  | 0xEF 0x01 [2]
            // addr   | cmd.address [4]
            // ident  | 0x01 [1]
            // length | 0x00 0x03 [2]
            // instr  | 0x1D [1]
            // chksum | checksum [2]
            Self::TemplateNum => {
                writer.write_cmd_bytes(&[0x01]);
                writer.write_cmd_bytes(&[0x00, 0x03]);
                writer.write_cmd_bytes(&[0x1D]);
            }

            // Required packet:
            // headr  | 0xEF 0x01 [2]
            // addr   | cmd.address [4]
            // ident  | 0x01 [1]
            // length | 0x00 0x03 [2]
            // instr  | 0x05 [1]
            // chksum | checksum [2]
            Self::RegModel => {
                writer.write_cmd_bytes(&[0x01]);
                writer.write_cmd_bytes(&[0x00, 0x03]);
                writer.write_cmd_bytes(&[0x05]);
            }

            // Required packet:
            // headr  | 0xEF 0x01 [2]
            // addr   | cmd.address [4]
            // ident  | 0x01 [1]
            // length | 0x00 0x06 [2]
            // instr  | 0x06 [1]
            // bufid  | buffer [1]
            // index  | index [2]
            // chksum | checksum [2]
            Self::Store { buffer, index } => {
                writer.write_cmd_bytes(&[0x01]);
                writer.write_cmd_bytes(&[0x00, 0x06]);
                writer.write_cmd_bytes(&[0x06]);
                writer.write_cmd_bytes(&[*buffer]);
                writer.write_cmd_bytes(&index.to_be_bytes()[..]);
            }

            // Required packet:
            // headr  | 0xEF 0x01 [2]
            // addr   | cmd.address [4]
            // ident  | 0x01 [1]
            // length | 0x00 0x07 [2]
            // instr  | 0x0c [1]
            // index  | index [2]
            // ndelet | num_to_delete [2]
            // chksum | checksum [2]
            Self::DeletChar { start_index, num_to_delete } => {
                writer.write_cmd_bytes(&[0x01]);
                writer.write_cmd_bytes(&[0x00, 0x07]);
                writer.write_cmd_bytes(&[0x0c]);
                writer.write_cmd_bytes(&start_index.to_be_bytes()[..]);
                writer.write_cmd_bytes(&num_to_delete.to_be_bytes()[..]);
            }
        }
    }
}
