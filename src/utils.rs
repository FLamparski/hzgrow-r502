/// Allows a type to define how to deserialise itself from some bytes
pub trait FromPayload {
    fn from_payload(payload: &[u8]) -> Self;
}

/// Something that lets you write commands (typically, a `R502`).
pub trait CommandWriter {
    fn write_cmd_bytes(&mut self, bytes: &[u8]);
}

/// Allows a type to define how to serialise itself into a CommandWriter.
///
/// This is implemented so that byte-level stuff can be kept out of the
/// main driver implementation body.
pub trait ToPayload {
    fn to_payload(&self, writer: &mut dyn CommandWriter);
}

/// Error type for low-level R502 operations. Wraps transport-level
/// errors as well.
///
/// `RXE` and `TXE` will be the `Error` type(s) of your serial port
/// implementation, as defined by `embedded_hal::serial::Read<u8>::Error`
/// and `embedded_hal::serial::Write<u8>::Error` respectively.
#[derive(Debug)]
pub enum Error<TXE, RXE> {
    /// Error writing data to the R502. The wrapped error should have more
    /// information as to what is causing this.
    WriteError(TXE),

    /// Error reading data from the R502. The wrapped error should have more
    /// information as to what is causing this.
    RecvReadError(RXE),

    /// _Something_ was received but it was too short to tell what it was.
    RecvPacketTooShort,

    /// There were no requests in flight when a packet was received.
    ///
    /// _This probably shouldn't happen at this point_
    RecvUnsolicitedReply,

    /// A packet of unexpected type was received instead of the reply.
    RecvWrongReplyType,
}
