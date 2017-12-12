// TODO: Eventually remove dependency on crate pnetlink and replace it with some netlink code written right here.
// Alternatively, maybe write a properly design netlink crate as well and depend on that.
// See Go's package https://github.com/vishvananda/netlink for inspiration
extern crate libc;
extern crate pnetlink;

use pnetlink::packet::netlink::NetlinkConnection;
use pnetlink::packet::route::link::{Links,Link, LinksIterator};
use pnetlink::packet::route::addr::{Addresses,Addr};

pub struct Interface(Link, IpAddrIterator);

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
        // TODO: investigate why get_hw_addr() will ever return None. If it's due to an error in underlying netlink APIs then we should return an error here
        Ok(self.0.get_hw_addr().and_then(|mac_addr| {
            Some(HardwareAddr::from_bytes([mac_addr.0, mac_addr.1, mac_addr.2, mac_addr.3, mac_addr.4, mac_addr.5]))
        }))
    }

    // TODO: this should also include anycast addresses they way golang implementatio does
    /// Get the adapter's ip addresses (unicast ip addresses)
    pub fn ip_addrs(&self) -> Result<impl Iterator<Item=IpAddr>, IfConfigError> { // TODO: Should we rename this to unicast_ip_addresses?
        Ok(self.1)
    }

    // pub fn ip_addrs_multicast(&self) -> impl Iterator<Item=IpAddr> { // TODO: Should we rename this to unicast_ip_addresses?
    //     IpAddrIterator::from(self.0.FistUnicastAddress)
    // }


    pub fn flags(&self) -> Flags {
        self.0.get_flags() & Flags::all()
    }
}

pub struct InterfaceIterator {
    netlink_connection: NetlinkConnection,
    links_iterator: Option<Box<LinksIterator<&mut NetlinkConnection>>>,
}


impl Iterator for InterfaceIterator {
    type Item = Interface;
    fn next(&mut self) -> Option<Interface> {
        if (self.links_iterator.is_none()) {
            self.links_iterator = Some(self.netlink_connection.links_iter())
        }
        
        match self.links_iterator.next() {
            Some(link) => {
                let ip_iter = self.conn.get_link_addrs(None, &link);
                Some(Interface(link, ip_iter))
            },
            None => None,
        }
    }
}

struct IpAddrIterator {
    net_link_addr_iterator: Box<Iterator<Item=Addr>>
}

impl IpAddrIterator {
    fn from(net_link_addr_iterator: Box<Iterator<Item=Addr>>) -> Self {
        Self { net_link_addr_iterator }
    }
}

impl Iterator for IpAddrIterator {
    type Item = IpAddr;
    fn next(&mut self) -> Option<IpAddr> {
        self.net_link_addr_iterator.next().and_then(|a| a.get_local_ip())
    }
}

pub fn get_interfaces() -> Result<impl Iterator<Item=Interface>, Error> {
     let mut conn = NetlinkConnection::new();
     InterfaceIterator(conn.iter_links())
}