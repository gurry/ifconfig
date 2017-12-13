mod bindings;

extern crate winapi;
extern crate widestring;

use std;
use std::ffi::CStr;
use self::winapi::{AF_UNSPEC, ERROR_SUCCESS, ERROR_BUFFER_OVERFLOW};
use self::widestring::WideCStr;
use socket2;
use std::net::IpAddr;
use data_types::{Flags, HardwareAddr, EUI48_LEN};
use self::bindings::*;
use super::super::IfConfigError;


const IF_TYPE_ETHERNET_CSMACD: u32 = 6;
const IF_TYPE_ISO88025_TOKENRING: u32 = 9;
const IF_TYPE_PPP: u32 = 23;
const IF_TYPE_SOFTWARE_LOOPBACK: u32 = 24;
const IF_TYPE_ATM: u32 = 37;
const IF_TYPE_IEEE80211: u32 = 71;
const IF_TYPE_TUNNEL: u32 = 131;
const IF_TYPE_IEEE1394: u32 = 144;


pub struct Interface(PIP_ADAPTER_ADDRESSES);

impl Interface {
    pub fn index(&self) -> u32 {
        let mut index = unsafe {(*self.0).__bindgen_anon_1.__bindgen_anon_1.IfIndex }; // Using ipV4 if index
        if index == 0 { // If ipv4 index was zero. As per MSDN this can happen if ipv4 is disabled on this interface
            index = unsafe { (*self.0).Ipv6IfIndex };
        }

        index
    }

    pub fn mtu(&self) -> Option<u32> { // TODO: what does a value of 0xffffffff mean for the MTU field in the inner struct
        Some(unsafe { (*self.0).Mtu })
    }

    pub fn name(&self) -> Result<&str, IfConfigError> {
        unsafe { CStr::from_ptr((*self.0).AdapterName) }.to_str()
                        .map_err(|_| IfConfigError::BadStringFormat { // TODO: Do not lose the underlying error Utf8Error here. Add it to the context of IfConfigError somehow
                            msg: "AdapterName could not be converted to &str".to_string()
                        })
    }

    // TODO: can we return OsStr here instead?
    pub fn friendly_name(&self) -> Result<String, IfConfigError> {
        // Must return an owned string here because there's no way to return a zero-copy &str type since Windows wide strings very different from Rust's utf8 strings
        unsafe { WideCStr::from_ptr_str((*self.0).FriendlyName) }.to_string()
                        .map_err(|_| IfConfigError::BadStringFormat { // TODO: Do not lose the underlying error Utf8Error here. Add it to the context of IfConfigError somehow
                            msg: "FriendlyName could not be converted to Rust String".to_string()
                        })
    }

    // TODO: can we return OsStr here instead?
    pub fn description(&self) -> Result<String, IfConfigError> {
        // Must return an owned string here because there's no way to return a zero-copy &str type since Windows wide strings very different from Rust's utf8 strings
        unsafe { WideCStr::from_ptr_str((*self.0).Description) }.to_string()
                        .map_err(|_| IfConfigError::BadStringFormat { // TODO: Do not lose the underlying error Utf8Error here. Add it to the context of IfConfigError somehow
                            msg: "Description could not be converted to Rust String".to_string()
                        })
    }

    pub fn hw_addr(&self) -> Result<Option<HardwareAddr>, IfConfigError> {
        unsafe {
            let len = (*self.0).PhysicalAddressLength as usize;
            if len == 0 {
                Ok(None)
            }
            else if len != EUI48_LEN {
                 Err(IfConfigError::BadHardwareAddr { len })
            }
            else {
                let ptr: *const u8 = (*self.0).PhysicalAddress.as_ptr();
                let bytes = *(ptr as *const [u8; 6]);
                Ok(Some(HardwareAddr::from_bytes(bytes)))
            }
         }
    }

    // TODO: this should also include anycast addresses they way golang implementatio does
    /// Get the adapter's ip addresses (unicast ip addresses)
    pub fn ip_addrs(&self) -> Result<impl Iterator<Item=IpAddr>, IfConfigError> { // TODO: Should we rename this to unicast_ip_addresses?
        Ok(IpAddrIterator::from(unsafe { (*self.0).FirstUnicastAddress }))
    }

    // pub fn ip_addrs_multicast(&self) -> impl Iterator<Item=IpAddr> { // TODO: Should we rename this to unicast_ip_addresses?
    //     IpAddrIterator::from(self.0.FistUnicastAddress)
    // }


    pub fn flags(&self) -> Flags {
        // Shamelessly copied from what the Golang people are doing.
        // There is also a comment that ideally the below info should come from MIB_IF_ROW2.AccessType. But go with this for now.
        unsafe {
            match (*self.0).IfType {
                IF_TYPE_ETHERNET_CSMACD | IF_TYPE_ISO88025_TOKENRING | IF_TYPE_IEEE80211 | IF_TYPE_IEEE1394 => {
                    Flags::BROADCAST | Flags::MULTICAST
                },
                IF_TYPE_PPP | IF_TYPE_TUNNEL => {
                    Flags::POINT_TO_POINT | Flags::MULTICAST
                },
                IF_TYPE_SOFTWARE_LOOPBACK => {
                    Flags::LOOPBACK | Flags::MULTICAST
                },
                IF_TYPE_ATM => {
                    Flags::BROADCAST | Flags::POINT_TO_POINT | Flags::MULTICAST // assume all services available; LANE, point-to-point and point-to-multipoint
                }
                _ => Flags::empty()
            }
        }
    }
}

pub struct InterfaceIterator {
    _adapter_addresses_buffer: Vec<u8>,
    current_ptr: PIP_ADAPTER_ADDRESSES,
}

impl InterfaceIterator {
    fn from(mut adapter_addresses_buffer: Vec<u8>) -> Self {
        let start_ptr: PIP_ADAPTER_ADDRESSES = unsafe { std::mem::transmute(adapter_addresses_buffer.as_mut_ptr()) };
        Self {_adapter_addresses_buffer: adapter_addresses_buffer, current_ptr: start_ptr }
    }
}

impl Iterator for InterfaceIterator {
    type Item = Interface;
    fn next(&mut self) -> Option<Interface> {
        if self.current_ptr != std::ptr::null_mut() {
            let interface = Interface(self.current_ptr);
            self.current_ptr = unsafe { (*self.current_ptr).Next } ;
            Some(interface)
        }
        else {
            None
        }
    }
}

struct IpAddrIterator {
    _adapter_unicast_ptr: PIP_ADAPTER_UNICAST_ADDRESS_LH,
    current_ptr: PIP_ADAPTER_UNICAST_ADDRESS_LH,
}

impl IpAddrIterator {
    fn from(adapter_unicast_ptr: PIP_ADAPTER_UNICAST_ADDRESS_LH) -> Self {
        Self { _adapter_unicast_ptr: adapter_unicast_ptr, current_ptr: adapter_unicast_ptr }
    }
}

impl Iterator for IpAddrIterator {
    type Item = IpAddr;
    fn next(&mut self) -> Option<IpAddr> {
        if self.current_ptr != std::ptr::null_mut() {
            let ip_addr = unsafe { socket_address_to_ipaddr(&(*self.current_ptr).Address) };
            self.current_ptr = unsafe { (*self.current_ptr).Next };
            Some(ip_addr)
        }
        else {
            None
        }
    }
}

unsafe fn socket_address_to_ipaddr(socket_address: &SOCKET_ADDRESS) -> IpAddr {
    let sockaddr = socket2::SockAddr::from_raw_parts(std::mem::transmute(socket_address.lpSockaddr), socket_address.iSockaddrLength);

    // Could be either ipv4 or ipv6
    sockaddr.as_inet()
        .map(|s| IpAddr::V4(*s.ip()))
        .unwrap_or_else(|| IpAddr::V6(*sockaddr.as_inet6().unwrap().ip()))
}

pub fn get_interfaces() -> Result<InterfaceIterator, IfConfigError> {
    unsafe {
        let mut buf_len: ULONG = 0;
        let result = GetAdaptersAddresses(AF_UNSPEC as u32, 0, std::ptr::null_mut(), std::ptr::null_mut(), &mut buf_len as *mut ULONG);

        assert!(result != ERROR_SUCCESS);

        // TODO: handle all other errors this function can return properly. See this for list of errors: https://msdn.microsoft.com/en-us/library/windows/desktop/aa365915(v=vs.85).aspx 
        if result != ERROR_BUFFER_OVERFLOW {
            return Err(IfConfigError::OsError { error_code: result }); // TODO: design proper error types and return that
        }

        let mut adapters_addresses_buffer: Vec<u8> = vec![0; buf_len as usize];
        let mut adapter_addresses_ptr: PIP_ADAPTER_ADDRESSES = std::mem::transmute(adapters_addresses_buffer.as_mut_ptr());
        let mut result = GetAdaptersAddresses(AF_UNSPEC as u32, 0, std::ptr::null_mut(), adapter_addresses_ptr, &mut buf_len as *mut ULONG);

        // Buffer overflowed again? Try once more, now with ~15K buffer as recommended on MSDN (unless buf_len requested is even larger)
        // (See https://msdn.microsoft.com/en-us/library/windows/desktop/aa365915(v=vs.85).aspx)
        if result == ERROR_BUFFER_OVERFLOW {
            const RECOMMENDED_BUF_LEN: u32 = 15000;
            buf_len  = std::cmp::max(RECOMMENDED_BUF_LEN, buf_len); 
            adapters_addresses_buffer = vec![0; buf_len as usize];
            adapter_addresses_ptr = std::mem::transmute(adapters_addresses_buffer.as_mut_ptr());
            result = GetAdaptersAddresses(AF_UNSPEC as u32, 0, std::ptr::null_mut(), adapter_addresses_ptr, &mut buf_len as *mut ULONG);
            if result != ERROR_SUCCESS {
                return Err(IfConfigError::OsError { error_code: result }); // TODO: design proper error types and return that
            }
        }
        else if result != ERROR_SUCCESS {
           return Err(IfConfigError::OsError { error_code: result }); // TODO: design proper error types and return that
        }

        Ok(InterfaceIterator::from(adapters_addresses_buffer))
    }
}