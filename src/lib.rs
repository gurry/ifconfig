
#[macro_use] extern crate foreign_types;
#[macro_use] extern crate failure;
#[macro_use] extern crate bitflags;

use std::io;
use std::ptr;
use std::ffi::CStr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use failure::Error;


#[cfg(target_os = "linux")]
mod linux {
    // TODO: Eventually remove dependency on crate pnetlink and replace it with some netlink code written right here.
    // Alternatively, maybe write a properly design netlink crate as well and depend on that.
    // See Go's package https://github.com/vishvananda/netlink for inspiration
    extern crate libc;
    extern crate pnetlink;
    use pnetlink::packet::netlink::NetlinkConnection;
    use pnetlink::packet::route::link::{Links,Link};
    use pnetlink::packet::route::addr::{Addresses,Addr};

    let mut conn = NetlinkConnection::new();
    let links = conn.iter_links().unwrap().collect::<Vec<_>>();
    for link in links {
        print_link(&link);
        for addr in conn.get_link_addrs(None, &link).unwrap() {
            //println!("{:?}", addr.get_ip());
            print_addr(&addr);
        }
    }




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

    fn get_addrs() -> Result<impl Iterator<Item=IpAddr>, Error> {
        let mut conn = NetlinkConnection::new();
        let links = conn.iter_links().unwrap().collect::<Vec<_>>();
        for link in links {
            print_link(&link);
            for addr in conn.get_link_addrs(None, &link).unwrap() {
                print_addr(&addr);
            }
        }
    }
}

#[cfg(target_os = "windows")]
mod windows {
    extern crate winapi;
    use winapi::{AF_UNSPEC, ERROR_SUCCESS, ERROR_BUFFER_OVERFLOW, ULONG};

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

    unsafe fn get_adapter(adapter_addresses_ptr: PIP_ADAPTER_ADDRESSES) -> Result<Adapter> {
        let adapter_addresses = &*adapter_addresses_ptr;
        let adapter_name = CStr::from_ptr(adapter_addresses.AdapterName).to_str()?.to_owned();
        let dns_servers = get_dns_servers(adapter_addresses.FirstDnsServerAddress)?;
        let unicast_addresses = get_unicast_addresses(adapter_addresses.FirstUnicastAddress)?;

        let description = WideCString::from_ptr_str(adapter_addresses.Description).to_string()?;
        let friendly_name = WideCString::from_ptr_str(adapter_addresses.FriendlyName).to_string()?;
        Ok(Adapter {
            adapter_name: adapter_name,
            ip_addresses: unicast_addresses,
            dns_servers: dns_servers,
            description: description,
            friendly_name: friendly_name,
        })
    }

    unsafe fn socket_address_to_ipaddr(socket_address: &SOCKET_ADDRESS) -> IpAddr {
        let sockaddr = socket2::SockAddr::from_raw_parts(std::mem::transmute(socket_address.lpSockaddr), socket_address.iSockaddrLength);

        // Could be either ipv4 or ipv6
        sockaddr.as_inet()
            .map(|s| IpAddr::V4(*s.ip()))
            .unwrap_or_else(|| IpAddr::V6(*sockaddr.as_inet6().unwrap().ip()))
    }

    unsafe fn get_dns_servers(mut dns_server_ptr: PIP_ADAPTER_DNS_SERVER_ADDRESS_XP) -> Result<Vec<IpAddr>> {
        let mut dns_servers = vec![];

        while dns_server_ptr != std::ptr::null_mut() {
            let dns_server = &*dns_server_ptr;
            let ipaddr = socket_address_to_ipaddr(&dns_server.Address);
            dns_servers.push(ipaddr);

            dns_server_ptr = dns_server.Next;
        }

        Ok(dns_servers)
    }

    unsafe fn get_unicast_addresses(mut unicast_addresses_ptr: PIP_ADAPTER_UNICAST_ADDRESS_LH) -> Result<Vec<IpAddr>> {
        let mut unicast_addresses = vec![];

        while unicast_addresses_ptr != std::ptr::null_mut() {
            let unicast_address = &*unicast_addresses_ptr;
            let ipaddr = socket_address_to_ipaddr(&unicast_address.Address);
            unicast_addresses.push(ipaddr);

            unicast_addresses_ptr = unicast_address.Next;
        }

        Ok(unicast_addresses)
    }

    fn get_addrs() -> Result<impl Iterator<Item=IpAddr>, Error> {
        // TODO: Call GetAdapters here or something
        unimplemented!()
    }
}

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
pub fn get_interfaces() -> Result<impl Iterator<Item=&Interface>, Error> {
    Ok(Vec::new().iter())
}

// Returns a list of the system's unicast interface addresses.
//
// The returned list does not identify the associated interface.
// Use get_interfaces and Interface.addrs() for more detail.
pub fn get_addrs() -> Result<impl Iterator<Item=IpAddr>, Error> {
    #[cfg(target_os = "linux")]
    linux::get_addrs()

    #[cfg(target_os = "windows")]
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
