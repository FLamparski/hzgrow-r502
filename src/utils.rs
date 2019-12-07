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
