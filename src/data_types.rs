
const MaxHwAddrLength: u32 = 8;
// IEEE MAC-48, EUI-48 and EUI-64 form
pub struct HardwareAddr {
    addr: Vec<u8>, // TODO: make this zero cast. Use a slice instead of vec here
}

// impl HardwareAddr {
//     fn from(bytes: &[u8]) -> Self {
//         Self { addr: bytes.clone(), len: bytes.len() }
//     }
// }

bitflags! {
    pub struct Flags: u32 {
        const UP              = 0b00000001; // Inteface is up
        const BROADCAST       = 0b00000010; // Interface supports broadcast
        const LOOPBACK        = 0b00000100; // Interface is loopback
        const POINT_TO_POINT  = 0b00001000; // Interface is part of a point-to-point link
        const MULTICAST       = 0b00010000; // Interface supports multicast
    }
}