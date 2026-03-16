pub trait ToLeBytes {
    fn to_le_bytes(&self) -> Vec<u8>;
}

impl ToLeBytes for Vec<u64> {
    fn to_le_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.len() * 8);
        for v in self {
            buf.extend_from_slice(&v.to_le_bytes());
        }

        buf
    }
} 