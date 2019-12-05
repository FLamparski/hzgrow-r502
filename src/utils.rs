pub trait FromPayload {
    fn from_payload(payload: &[u8]) -> Self;
}

pub trait CommandWriter {
    fn write_cmd_bytes(&mut self, bytes: &[u8]);
}

pub trait ToPayload {
    fn to_payload(&self, writer: &mut dyn CommandWriter);
}
