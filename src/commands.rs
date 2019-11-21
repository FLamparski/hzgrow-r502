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

    /// Performs a handshake with the device to verify the password.
    /// The default password on the R502 is 0x00000000.
    VfyPwd {
        /// Device address, in case you have many of them connected to the same UART.
        /// The default is 0xFFFFFFFF
        address: u32,

        /// The device password.
        password: u32,
    },
}
