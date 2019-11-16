//# Naming conventions etc follow the R502 datasheet, see:
//# https://www.dropbox.com/sh/epucei8lmoz7xpp/AAAmon04b1DiSOeh1q4nAhzAa?dl=0&preview=R502+fingerprint+module+user+manual-V1.2.pdf

/// Enum for commands one can send to the R502. Names match the datasheet.
#[derive(Debug)]
pub enum Command {
    /// Reads system status and basic configuration
    ReadSysPara {
        /// Device address, in case you have many of them connected to the same UART.
        /// The default is 0xFFFFFFFF
        address: u32,
    },
}

/// Responses to commands returned by the R502. Names are the same as commands.
#[derive(Debug)]
pub enum Reply {
    /// Contains system status and configuration information
    ReadSysPara {
        address: u32,
        confirmation_code: u8,
        system_parameters: SystemParameters,
        checksum: u16,
    },
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
