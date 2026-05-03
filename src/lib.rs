pub struct Memory {
    pub bytes: [u8; 4 * 1024],
}

impl Memory {
    pub fn new() -> Self {
        Self { 
            bytes: [0; 4 * 1024],
        }
    }

    pub fn access(&self, offset: u16) -> u8 {
        return self.bytes[offset as usize];
    }
}
