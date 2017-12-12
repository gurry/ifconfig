pub const EUI48_LEN: usize = 6;

// IEEE MAC-48, EUI-48 and EUI-64 form
pub struct HardwareAddr {
    // TODO: should we be carrying a reference to the byte array instead of a copy? Will that be faster?
    bytes: [u8; EUI48_LEN], // TODO: Should we also cater for 8 byte EUI-64 addresses?
}

impl HardwareAddr {
    pub fn from_bytes(bytes: [u8; EUI48_LEN]) -> Self {
        Self { bytes }
    }

    // TODO: Should we also cater for 8 byte EUI-64 addresses?
    pub fn to_string(&self) -> String {
        format!("{:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}", self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3], self.bytes[4], self.bytes[5])
    }
}

bitflags! {
    pub struct Flags: u32 {
        const UP              = 0b00000001; // Inteface is up
        const BROADCAST       = 0b00000010; // Interface supports broadcast
        const LOOPBACK        = 0b00000100; // Interface is loopback
        const POINT_TO_POINT  = 0b00001000; // Interface is part of a point-to-point link
        const MULTICAST       = 0b00010000; // Interface supports multicast
    }
}