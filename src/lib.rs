#[macro_use] extern crate failure;
#[macro_use] extern crate bitflags;

use std::io;
use std::ptr;
use std::ffi::CStr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use failure::Error;

// IEEE MAC-48, EUI-48 and EUI-64 form
struct HardwareAddr([u8; 6]); // TODO: should we use something other than bytes as the underlying type? Are 6 bytes enough?

struct Interface {
  	index: u32,
  	mtu: u32,    // TODO; should we use Bytes() type here form byte_units crate
  	name: String, // TODO: should we use an &str instead?
  	hw_addr: HardwareAddr, 
  	flags: Flags,
}

bitflags! {
    struct Flags: u32 {
        const FLAG_UP              = 0b00000001;
        const FLAG_BROADCAST       = 0b00000010;
        const FLAG_LOOPBACK        = 0b00000100;
        const FLAG_POINTTOPOINT    = 0b00001000;
        const FLAG_MULTICAST       = 0b00010000;
    }
}

// Returns a list of the system's network interfaces.
#[cfg(target_os = "linux")]
pub fn get_interfaces() -> Result<impl Iterator<Item=&Interface>, Error> {
    Ok(Vec::new().iter())
}

#[cfg(target_os = "windows")]
pub fn get_interfaces() -> Result<impl Iterator<Item=&Interface>, Error> {
    Ok(Vec::new().iter())
}

// Returns a list of the system's unicast interface addresses.
//
// The returned list does not identify the associated interface.
// Use get_interfaces and Interface.addrs() for more detail.
#[cfg(target_os = "linux")]
pub fn get_addrs() -> Result<impl Iterator<Item=IpAddr>, Error> {
    linux::get_addrs()
}

#[cfg(target_os = "windows")]
pub fn get_addrs() -> Result<impl Iterator<Item=IpAddr>, Error> {
    windows::get_addrs()
}

// Corresponding functions from golang recorded below for reference
// func Interfaces() ([]Interface, error) {
// 	ift, err := interfaceTable(0)
// 	if err != nil {
// 		return nil, &OpError{Op: "route", Net: "ip+net", Source: nil, Addr: nil, Err: err}
// 	}
// 	if len(ift) != 0 {
// 		zoneCache.update(ift)
// 	}
// 	return ift, nil
// }

// // InterfaceAddrs returns a list of the system's unicast interface
// // addresses.
// //
// // The returned list does not identify the associated interface; use
// // Interfaces and Interface.Addrs for more detail.
// func InterfaceAddrs() ([]Addr, error) {
// 	ifat, err := interfaceAddrTable(nil)
// 	if err != nil {
// 		err = &OpError{Op: "route", Net: "ip+net", Source: nil, Addr: nil, Err: err}
// 	}
// 	return ifat, err
// }
