#![feature(conservative_impl_trait)]

#[macro_use] extern crate failure;
#[macro_use] extern crate bitflags;
extern crate socket2;


mod os;
mod data_types;

#[cfg(target_os = "linux")]
use os::linux as imp;

#[cfg(target_os = "windows")]
use os::windows as imp;

pub use imp::Interface;

bitflags! {
    struct Flags: u32 {
        const FLAG_UP              = 0b00000001;
        const FLAG_BROADCAST       = 0b00000010;
        const FLAG_LOOPBACK        = 0b00000100;
        const FLAG_POINTTOPOINT    = 0b00001000;
        const FLAG_MULTICAST       = 0b00010000;
    }
}

#[derive(Debug, Fail)]
pub enum IfConfigError {
    #[fail(display = "{}", msg)]
    UnderlyingApiFailed {
        msg: String
    },

    #[fail(display = "OS error: {}", error_code)]
    OsError {
        error_code: u32,
    },

    #[fail(display = "Hardware addr has unsupported length {}", len)]
    BadHardwareAddr {
        len: usize,
    },

    #[fail(display = "No Hardware addr found")]
    HardwareAddrError,

    #[fail(display = "{}", msg)]
    BadStringFormat {
        msg: String
    },

    #[fail(display = "{}", msg)]
    ValueNotFound {
        msg: String
    },
}

// Returns a list of the system's network interfaces.
pub fn get_interfaces() -> Result<impl Iterator<Item=Interface>, IfConfigError> {
    imp::get_interfaces()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_not_panic() {
        let interfaces = get_interfaces().unwrap();
        for i in interfaces {
            let index = i.index();
            let name = i.name();
            let friendly_name = i.friendly_name();
            let mtu = i.mtu();
            let description = i.description ();
            let ip_addr: Vec<String> = i.ip_addrs().unwrap().map(|ip| ip.to_string()).collect();
            let hw_addr = i.hw_addr();
        }
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
