
#[macro_use] extern crate foreign_types;
#[macro_use] extern crate failure;
extern crate winapi;


use foreign_types::{ForeignType, ForeignTypeRef};
use std::io;
use std::ptr;
use std::ffi::CStr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use failure::Error;
use winapi::{AF_UNSPEC, ERROR_SUCCESS, ERROR_BUFFER_OVERFLOW, ULONG};



#[cfg(target_os = "linux")]
mod linux {
    extern crate libc;
    foreign_type! {
        type CType = libc::ifaddrs;
        fn drop = libc::freeifaddrs;
        pub struct IfAddrs;
        pub struct IfAddrsRef;
    }

    impl IfAddrs {
        pub fn get() -> io::Result<IfAddrs> {
            unsafe {
                let mut ifaddrs = ptr::null_mut();
                let r = libc::getifaddrs(&mut ifaddrs);
                if r == 0 {
                    Ok(IfAddrs::from_ptr(ifaddrs))
                } else {
                    Err(io::Error::last_os_error())
                }
            }
        }
    }

    impl IfAddrsRef {
        pub fn next(&self) -> Option<&IfAddrsRef> {
            unsafe {
                let next = (*self.as_ptr()).ifa_next;
                if next.is_null() {
                    None
                } else {
                    Some(IfAddrsRef::from_ptr(next))
                }
            }
        }

        pub fn name(&self) -> &str {
            unsafe {
                let s = CStr::from_ptr((*self.as_ptr()).ifa_name);
                s.to_str().unwrap()
            }
        }

        pub fn addr(&self) -> Option<IpAddr> {
            unsafe {
                let addr = (*self.as_ptr()).ifa_addr;
                if addr.is_null() {
                    return None;
                }

                match (*addr).sa_family as _ {
                    libc::AF_INET => {
                        let addr = addr as *mut libc::sockaddr_in;
                        // It seems like this to_be shouldn't be needed?
                        let addr = Ipv4Addr::from((*addr).sin_addr.s_addr.to_be());
                        Some(IpAddr::V4(addr))
                    }
                    libc::AF_INET6 => {
                        let addr = addr as *mut libc::sockaddr_in6;
                        let addr = Ipv6Addr::from((*addr).sin6_addr.s6_addr);
                        Some(IpAddr::V6(addr))
                    }
                    _ => None,
                }
            }
        }

        pub fn iter<'a>(&'a self) -> Iter<'a> {
            Iter(Some(self))
        }
    }

    impl<'a> IntoIterator for &'a IfAddrs {
        type Item = &'a IfAddrsRef;
        type IntoIter = Iter<'a>;

        fn into_iter(self) -> Iter<'a> {
            self.iter()
        }
    }

    impl<'a> IntoIterator for &'a IfAddrsRef {
        type Item = &'a IfAddrsRef;
        type IntoIter = Iter<'a>;

        fn into_iter(self) -> Iter<'a> {
            self.iter()
        }
    }

    pub struct Iter<'a>(Option<&'a IfAddrsRef>);

    impl<'a> Iterator for Iter<'a> {
        type Item = &'a IfAddrsRef;

        fn next(&mut self) -> Option<&'a IfAddrsRef> {
            let cur = match self.0 {
                Some(cur) => cur,
                None => return None,
            };

            self.0 = cur.next();
            Some(cur)
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn basic() {
            let addrs = IfAddrs::get().unwrap();
            println!(
                "{:?}",
                addrs
                    .iter()
                    .map(|a| (a.name(), a.addr()))
                    .collect::<Vec<_>>()
            );
        }
    }
}

#[cfg(target_os = "windows")]
mod windows {
fn GetAdaptersAddresses ( Family : ULONG , Flags : ULONG , Reserved : PVOID , AdapterAddresses : PIP_ADAPTER_ADDRESSES , SizePointer : PULONG , ) -> ULONG; 

pub fn get_adapters() -> Result<Vec<Adapter>> {
    unsafe {
        let mut buf_len: ULONG = 0;
        let result = GetAdaptersAddresses(AF_UNSPEC as u32, 0, std::ptr::null_mut(), std::ptr::null_mut(), &mut buf_len as *mut ULONG);

        assert!(result != ERROR_SUCCESS);

        if result != ERROR_BUFFER_OVERFLOW {
            bail!(ErrorKind::Os(result));
        }

        let mut adapters_addresses_buffer: Vec<u8> = vec![0; buf_len as usize];
        let mut adapter_addresses_ptr: PIP_ADAPTER_ADDRESSES = std::mem::transmute(adapters_addresses_buffer.as_mut_ptr());
        let result = GetAdaptersAddresses(AF_UNSPEC as u32, 0, std::ptr::null_mut(), adapter_addresses_ptr, &mut buf_len as *mut ULONG);

        if result != ERROR_SUCCESS {
            bail!(ErrorKind::Os(result));
        }

        let mut adapters = vec![];
        while adapter_addresses_ptr != std::ptr::null_mut() {
            adapters.push(get_adapter(adapter_addresses_ptr)?);
            adapter_addresses_ptr = (*adapter_addresses_ptr).Next;
        }

        Ok(adapters)
    }
}

}

// IEEE MAC-48, EUI-48 and EUI-64 form
struct HardwareAddr(u8[]); // TODO: should we use something other than bytes as the underlying type?

struct Interface {
  	index: u32,
  	mtu: u32,    // TODO; should we use Bytes() type here form byte_units crate
  	name: String, // TODO: should we use an &str instead?
  	hw_addr: HardwareAddr, 
  	flags: Flags,
}

struct Flags(u32); // TODO: should we use u32 as the underlying type?
  
  const (
  	FlagUp           Flags = 1 << iota // interface is up
  	FlagBroadcast                      // interface supports broadcast access capability
  	FlagLoopback                       // interface is a loopback interface
  	FlagPointToPoint                   // interface belongs to a point-to-point link
  	FlagMulticast                      // interface supports multicast access capability
  )

pub fn get_interfaces() -> Result<impl Iterator<Item=&Interface>, Error> {
    Ok(Vec::new().iter())
}