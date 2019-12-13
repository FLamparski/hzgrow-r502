use crate::utils::FromPayload;
use byteorder::{BigEndian, ByteOrder};

/// Responses to commands returned by the R502. Names are the same as commands.
#[derive(Debug)]
pub enum Reply {
    /// Contains system status and configuration information
    ReadSysPara(ReadSysParaResult),

    /// Contains result of password verification
    VfyPwd(VfyPwdResult),

    /// Contains result of acquiring an image
    GenImg(GenImgResult),

    /// Contains result of processing a fingerprint image into a _character buffer_
    Img2Tz(Img2TzResult),

    /// Contains result of searching the library for a match
    Search(SearchResult),

    /// Contains result of loading a character file into a character buffer
    LoadChar(LoadCharResult),

    /// Contains result of matching two fingers against each other
    Match(MatchResult),
}

/// Result struct for the `ReadSysPara` call
#[derive(Debug)]
pub struct ReadSysParaResult {
    /// Address of the R502 this message came from
    pub address: u32,

    /// Status code
    pub confirmation_code: u8,

    /// System parameters
    pub system_parameters: SystemParameters,

    pub checksum: u16,
}

impl FromPayload for ReadSysParaResult {
    // Expected packet:
    // headr  | 0xEF 0x01 [2]
    // addr   | cmd.address [4]
    // ident  | 0x01 [1]
    // length | 0x00 0x03 [2] == 19 (3 + 16)
    // confrm | 0x0F [1]
    // params | (params) [16]
    // chksum | checksum [2]
    fn from_payload(payload: &[u8]) -> Self {
        return Self {
            address: BigEndian::read_u32(&payload[2..6]),
            confirmation_code: payload[9],
            checksum: BigEndian::read_u16(&payload[26..28]),
            system_parameters: SystemParameters::from_payload(&payload[10..26]),
        };
    }
}

/// Result struct for the `VfyPwd` call
#[derive(Debug)]
pub struct VfyPwdResult {
    /// Address of the R502 this message came from
    pub address: u32,

    /// Handshake result
    pub confirmation_code: PasswordVerificationState,

    pub checksum: u16,
}

impl FromPayload for VfyPwdResult {
    fn from_payload(payload: &[u8]) -> Self {
        return Self {
            address: BigEndian::read_u32(&payload[2..6]),
            confirmation_code: PasswordVerificationState::from(payload[9]),
            checksum: BigEndian::read_u16(&payload[10..12]),
        };
    }
}

/// Result struct for the `GenImg` call
#[derive(Debug)]
pub struct GenImgResult {
    /// Address of the R502 that sent this message
    pub address: u32,

    /// Fingerprint capture result
    pub confirmation_code: GenImgStatus,

    pub checksum: u16,
}

impl FromPayload for GenImgResult {
    fn from_payload(payload: &[u8]) -> Self {
        return Self {
            address: BigEndian::read_u32(&payload[2..6]),
            confirmation_code: GenImgStatus::from(payload[9]),
            checksum: BigEndian::read_u16(&payload[10..12]),
        };
    }
}

/// Result struct for the `Img2Tz` struct
#[derive(Debug)]
pub struct Img2TzResult {
    /// Address of the R502 that sent this message
    pub address: u32,

    /// Fingerprint processing result
    pub confirmation_code: Img2TzStatus,

    pub checksum: u16,
}

impl FromPayload for Img2TzResult {
    fn from_payload(payload: &[u8]) -> Self {
        return Self {
            address: BigEndian::read_u32(&payload[2..6]),
            confirmation_code: Img2TzStatus::from(payload[9]),
            checksum: BigEndian::read_u16(&payload[10..12]),
        };
    }
}

/// Result struct for the `Search` call
#[derive(Debug)]
pub struct SearchResult {
    /// Address of the R502 that sent this message
    pub address: u32,

    /// Search processing result
    pub confirmation_code: SearchStatus,

    /// Index, in the library, of the best match.
    pub match_id: u16,

    /// Match score
    ///
    /// **Note:** A match score of 0 means no match (`confirmation_code` will be `NoMatch`)
    pub match_score: u16,

    pub checksum: u16,
}

impl FromPayload for SearchResult {
    fn from_payload(payload: &[u8]) -> Self {
        return Self {
            address: BigEndian::read_u32(&payload[2..6]),
            confirmation_code: SearchStatus::from(payload[9]),
            match_id: BigEndian::read_u16(&payload[10..12]),
            match_score: BigEndian::read_u16(&payload[12..14]),
            checksum: BigEndian::read_u16(&payload[14..16]),
        };
    }
}

/// Structure containing the status code of the `LoadChar` call
#[derive(Debug)]
pub struct LoadCharResult {
    /// Address of the R502 that sent this message
    pub address: u32,

    /// Response code
    pub confirmation_code: LoadCharStatus,

    pub checksum: u16,
}

impl FromPayload for LoadCharResult {
    fn from_payload(payload: &[u8]) -> Self {
        return Self {
            address: BigEndian::read_u32(&payload[2..6]),
            confirmation_code: LoadCharStatus::from(payload[9]),
            checksum: BigEndian::read_u16(&payload[10..12]),
        };
    }
}

/// Structure containing the status code of the `Match` call
#[derive(Debug)]
pub struct MatchResult {
    /// Address of the R502 that sent this message
    pub address: u32,

    /// Response code
    pub confirmation_code: MatchStatus,

    /// Match confidence value
    pub match_score: u16,

    pub checksum: u16,
}

impl FromPayload for MatchResult {
    fn from_payload(payload: &[u8]) -> Self {
        return Self {
            address: BigEndian::read_u32(&payload[2..6]),
            confirmation_code: MatchStatus::from(payload[9]),
            match_score: BigEndian::read_u16(&payload[10..12]),
            checksum: BigEndian::read_u16(&payload[12..14]),
        };
    }
}

/// System status and configuration.
#[derive(Debug)]
pub struct SystemParameters {
    /// Status information. Use instance methods of SystemParameters to get to individual bits.
    pub status_register: u16,

    /// System identifier code, whatever that means - datasheet says this has a constant value of
    /// 0x0009
    pub system_identifier_code: u16,

    /// Finger library size (maximum, not the number of fingerprints enrolled)
    pub finger_library_size: u16,

    /// Security level [1-5]
    pub security_level: u16,

    /// Device address, repeated from the packet header
    pub device_address: u32,

    /// Packet size. Actually a size code [0-3]:\
    /// 0 = 32 bytes\
    /// 1 = 64 bytes\
    /// 2 = 128 bytes (the default)\
    /// 3 = 256 bytes
    pub packet_size: u16,

    /// Baud setting. To get actual baud value, multiply by 9600.
    ///
    /// Note, the datasheet contradicts itself as to what's the maximum baud rate supported by
    /// the device, and consequently what's the maximum here. In one place, it says the range is
    /// [1-6], in another it states the max baud rate is 115,200 giving [1-12].
    /// The default value is 6 for 57,600‬ baud.
    pub baud_setting: u16,
}

/// Convenience methods for reading fields of the R502's status register
impl SystemParameters {
    /// True if the R502 is busy executing another command.
    ///
    /// *Busy* in the datasheet.
    pub fn busy(self) -> bool {
        return self.status_register & (1u16 << 0) != 0;
    }

    /// True if the module found a matching finger - however you should
    /// always check the response to the actual matching request.
    ///
    /// *Pass* in the datasheet.
    pub fn has_finger_match(self) -> bool {
        return self.status_register & (1u16 << 1) != 0;
    }

    /// True if the password given in the handshake is correct.
    ///
    /// *PWD* in the datasheet.
    pub fn password_ok(self) -> bool {
        return self.status_register & (1u16 << 2) != 0;
    }

    /// True if the image buffer contains a valid image.
    ///
    /// *ImgBufStat* in the datasheet. Note that this method may return `false`
    /// and yet the R502 would still function and perform matches.
    pub fn has_valid_image(self) -> bool {
        return self.status_register & (1u16 << 3) != 0;
    }
}

impl FromPayload for SystemParameters {
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

/// Enum for the password handshake result
#[derive(Debug)]
pub enum PasswordVerificationState {
    Correct,
    Incorrect,
    Error,
}

impl PasswordVerificationState {
    pub fn from(byte: u8) -> Self {
        return match byte {
            0x00 => Self::Correct,
            0x13 => Self::Incorrect,
            0x01 => Self::Error,
            _ => panic!("Invalid VfyPwdResult: {:02x}", byte),
        };
    }
}

/// Enum for the `GenImg` status code
#[derive(Debug)]
pub enum GenImgStatus {
    /// Fingerprint has been captured successfully
    Success,

    /// Error reading packet from the host
    PacketError,

    /// Finger not detected
    FingerNotDetected,

    /// Image failed to capture
    ImageNotCaptured,
}

impl GenImgStatus {
    pub fn from(byte: u8) -> Self {
        return match byte {
            0x00 => Self::Success,
            0x01 => Self::PacketError,
            0x02 => Self::FingerNotDetected,
            0x03 => Self::ImageNotCaptured,
            _ => panic!("Invalid GenImgStatus: {:02x}", byte),
        };
    }
}

/// Enum for the `Img2Tz` status code
#[derive(Debug)]
pub enum Img2TzStatus {
    /// Fingerprint processed successfully
    Success,

    /// Error reading packet from the host
    PacketError,

    /// Fingerprint image overly distorted
    FingerprintImageDistorted,

    /// Could not process the fingerprint image. The original datasheet helpfully states:
    ///
    /// > fail to generate character file due to lackness of character point or over-smallness of fingerprint image
    ProcessingFailed,

    /// Input image buffer not valid
    InvalidInput,
}

impl Img2TzStatus {
    pub fn from(byte: u8) -> Self {
        return match byte {
            0x00 => Self::Success,
            0x01 => Self::PacketError,
            0x06 => Self::FingerprintImageDistorted,
            0x07 => Self::ProcessingFailed,
            0x15 => Self::InvalidInput,
            _ => panic!("Invalid Img2TzStatus: {:02x}", byte),
        };
    }
}

/// Enum for the `Search` status code
#[derive(Debug)]
pub enum SearchStatus {
    /// There is a match
    Success,
    /// Error reading packet from the host
    PacketError,
    /// No match - index and score will be 0
    NoMatch,
}

impl SearchStatus {
    pub fn from(byte: u8) -> Self {
        return match byte {
            0x00 => Self::Success,
            0x01 => Self::PacketError,
            0x09 => Self::NoMatch,
            _ => panic!("Invalid SearchStatus: {:02x}", byte),
        };
    }
}

/// `LoadChar` status code
#[derive(Debug)]
pub enum LoadCharStatus {
    /// Operation completed successfully.
    Success,
    /// Error reading packet from the host
    PacketError,
    /// Error reading the fingerprint file from the library:
    ///
    /// > error when reading template from library or the read out template is invalid
    LibraryReadError,
    /// Index given is out of range (eg. > 200 for the R502)
    IndexOutOfRange,
}

impl LoadCharStatus {
    pub fn from(byte: u8) -> Self {
        return match byte {
            0x00 => Self::Success,
            0x01 => Self::PacketError,
            0x0c => Self::LibraryReadError,
            0x0b => Self::IndexOutOfRange,
            _ => panic!("Invalid LoadCharStatus: {:02x}", byte),
        };
    }
}

/// `Match` status code
#[derive(Debug)]
pub enum MatchStatus {
    /// Match performed successfully and the two buffers match
    Success,
    /// Error reading packet from the host
    PacketError,
    /// Matching was performed but the two buffers don't match
    NoMatch,
}

impl MatchStatus {
    fn from(byte: u8) -> Self {
        return match byte {
            0x00 => Self::Success,
            0x01 => Self::PacketError,
            0x08 => Self::NoMatch,
            _ => panic!("Invalid MatchStatus: {:02x}", byte),
        };
    }
}
