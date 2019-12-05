/// Responses to commands returned by the R502. Names are the same as commands.
#[derive(Debug)]
pub enum Reply {
    /// Contains system status and configuration information
    ReadSysPara(ReadSysParaResult),

    VfyPwd(VfyPwdResult),

    GenImg(GenImgResult),
}

#[derive(Debug)]
pub struct ReadSysParaResult {
    pub address: u32,
    pub confirmation_code: u8,
    pub system_parameters: SystemParameters,
    pub checksum: u16,
}

#[derive(Debug)]
pub struct VfyPwdResult {
    pub address: u32,
    /// Handshake result
    pub confirmation_code: PasswordVerificationState,
    pub checksum: u16,
}

#[derive(Debug)]
pub struct GenImgResult {
    pub address: u32,
    pub confirmation_code: GenImgStatus,
    pub checksum: u16,
}

/// System status and configuration.
#[derive(Debug)]
pub struct SystemParameters {
    /// Status information. Use instance methods of SystemParameters to get to individual bits.
    pub status_register: u16,

    /// System identifier code, whatever that means - datasheet says this has a constant value of
    /// 0x0009
    pub system_identifier_code: u16,

    /// Finger library size.
    pub finger_library_size: u16,

    /// Security level [1-5]
    pub security_level: u16,

    /// Device address, in case you forgot, but then you'd need the device address to send it the
    /// `ReadSysPara` command... 🤔
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
    /// *ImgBufStat* in the datasheet.
    pub fn has_valid_image(self) -> bool {
        return self.status_register & (1u16 << 3) != 0;
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

#[derive(Debug)]
pub enum GenImgStatus {
    Success,
    PacketError,
    FingerNotDetected,
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
