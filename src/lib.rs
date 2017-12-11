#![feature(conservative_impl_trait)]

#[macro_use] extern crate failure;
#[macro_use] extern crate bitflags;
extern crate winapi;
extern crate widestring;
extern crate socket2;

use std::io;
use std::ptr;
use std::ffi::CStr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use failure::Error;

mod os;
mod data_types;

#[cfg(target_os = "linux")]
use os::linux as imp;

#[cfg(target_os = "windows")]
use os::windows as imp;

#[cfg(target_os = "windows")]
pub use os::windows::Interface as Interface;
#[cfg(target_os = "windows")]
use os::windows::InterfaceIterator;

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
pub fn get_interfaces() -> Result<impl Iterator<Item=Interface>, Error> {
    imp::get_interfaces()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let interfaces = get_interfaces().unwrap();
        for i in interfaces {
            println!("{:?}", i.index());
            println!("{:?}", i.name());
            println!("{:?}", i.mtu());
        }
        // println!(
        //     "{:?}",
        //     addrs
        //         .iter()
        //         .map(|a| (a.name(), a.addr()))
        //         .collect::<Vec<_>>()
        // );
    }
}

// Returns a list of the system's unicast interface addresses.
//
// The returned list does not identify the associated interface.
// Use get_interfaces and Interface.addrs() for more detail.
// #[cfg(target_os = "linux")]
// pub fn get_addrs() -> Result<impl Iterator<Item=IpAddr>, Error> {
//     os::linux::get_addrs()
// }

// #[cfg(target_os = "windows")]
// pub fn get_addrs() -> Result<impl Iterator<Item=IpAddr>, Error> {
//     os::windows::get_addrs()
// }

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
