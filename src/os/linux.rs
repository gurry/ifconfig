// TODO: Eventually remove dependency on crate pnetlink and replace it with some netlink code written right here.
// Alternatively, maybe write a properly design netlink crate as well and depend on that.
// See Go's package https://github.com/vishvananda/netlink for inspiration
extern crate libc;
extern crate pnetlink;

use pnetlink::packet::netlink::NetlinkConnection;
use pnetlink::packet::route::link::{Links,Link, LinksIterator};
use pnetlink::packet::route::addr::{Addresses,Addr};

pub struct Interface(Link);

impl Interface {
    pub fn index(&self) -> u32 {
        self.0.get_index()
    }

    pub fn mtu(&self) -> Option<u32> { // TODO: what does a value of 0xffffffff mean for the MTU field in the inner struct
        self.0.get_mtu()
    }

    pub fn name(&self) -> Result<&str, IfConfigError> {
        // TODO: the underlying netlink library is doing unwrap()s inside the get_name() function. We should ideally do netlink calls ourselves
        self.0.get_name().ok_or(IfConfigError::ValueNotFound { msg: "Underlying library returned no value for name"})
    }

    // TODO: can we return OsStr here instead?
    pub fn friendly_name(&self) -> Result<String, IfConfigError> {
        Ok("".to_string()) // Linux seems to have no concept of friendly name
    }

    // TODO: can we return OsStr here instead?
    pub fn description(&self) -> Result<String, IfConfigError> {
        Ok("".to_string()) // Linux seems to have no concept of description
    }

    pub fn hw_addr(&self) -> Result<Option<HardwareAddr>, IfConfigError> {
        self.0.get_hw_addr().and_then(|mac_addr| {
            Some(HardwareAddr::from_bytes([mac_addr.0, mac_addr.1, mac_addr.2, mac_addr.3, mac_addr.4, mac_addr.5]))
        }).ok_or(IfConfigError::HardwareAddrError)
    }

    // TODO: this should also include anycast addresses they way golang implementatio does
    /// Get the adapter's ip addresses (unicast ip addresses)
    pub fn ip_addrs(&self) -> impl Iterator<Item=IpAddr> { // TODO: Should we rename this to unicast_ip_addresses?
        IpAddrIterator::from(unsafe { (*self.0).FirstUnicastAddress })
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
    links_iterator: Box<LinksIterator<&mut Self>>
}


impl Iterator for InterfaceIterator {
    type Item = Interface;
    fn next(&mut self) -> Option<Interface> {
        self.links_iterator.next().and_then(|l| Some(Interface(l)))
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

pub fn get_interfaces() -> Result<impl Iterator<Item=Interface>, Error> {
    unimplemented!()
    // let mut conn = NetlinkConnection::new();
    // let links = conn.iter_links().unwrap().collect::<Vec<_>>();
}