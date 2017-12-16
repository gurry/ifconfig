// TODO: Eventually remove dependency on crate pnetlink and replace it with some netlink code written right here.
// Alternatively, maybe write a properly design netlink crate as well and depend on that.
// See Go's package https://github.com/vishvananda/netlink for inspiration
extern crate pnetlink;

use self::pnetlink::packet::netlink::NetlinkConnection;
use self::pnetlink::packet::route::link::{Links,Link, LinksIterator};
use self::pnetlink::packet::route::addr::{Addresses,Addr};

use std::net::IpAddr;
use std::io;
use std::vec::IntoIter;

use data_types::{Flags, HardwareAddr, IpAddrSet};
use super::super::IfConfigError;

pub struct Interface {
    link: Link, 
    ip_addrs: Vec<IpAddrSet>,
    name: String
}

impl Interface {
    fn from(link: Link, ip_addrs: Vec<IpAddrSet>) -> Result<Self, IfConfigError> {
        let name = link.get_name();
        if name.is_none() {
            return Err(IfConfigError::ValueNotFound { msg: "Underlying library returned no value for name".to_string() });
        }
        Ok(Self { link, ip_addrs, name: name.unwrap() })
    }
    pub fn index(&self) -> u32 {
        self.link.get_index()
    }

    pub fn mtu(&self) -> Option<u32> { // TODO: what does a value of 0xffffffff mean for the MTU field in the inner struct
        self.link.get_mtu()
    }

    pub fn name(&self) -> Result<&str, IfConfigError> {
        Ok(self.name.as_str())
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
        Ok(self.link.get_hw_addr().and_then(|mac_addr| {
            Some(HardwareAddr::from_bytes([mac_addr.0, mac_addr.1, mac_addr.2, mac_addr.3, mac_addr.4, mac_addr.5]))
        }))
    }

    // TODO: this should also include anycast addresses they way golang implementatio does
    /// Get the adapter's ip addresses (unicast ip addresses)
    pub fn ip_addrs(&self) -> Result<impl Iterator<Item=IpAddrSet>, IfConfigError> { // TODO: Should we rename this to unicast_ip_addresses?
        Ok(IpAddrSetIterator { addrs: self.ip_addrs.clone().into_iter() })
    }

    // pub fn ip_addrs_multicast(&self) -> impl Iterator<Item=IpAddr> { // TODO: Should we rename this to unicast_ip_addresses?
    //     IpAddrIterator::from(self.0.FistUnicastAddress)
    // }


    pub fn flags(&self) -> Flags {
        Flags::from_bits(self.link.get_flags().bits() & Flags::all().bits()).expect("Creation of flags cannot faile")
    }
}

fn to_interfaces(mut conn: NetlinkConnection) -> Result<IntoIter<Interface>, IfConfigError> {
    // let req = NetlinkRequestBuilder::new(RTM_GETLINK, NLM_F_DUMP)
    //     .append(
    //         IfInfoPacketBuilder::new()
    //             .build()
    //     ).build();
    // try!(conn.write(req.packet()));
    // let reader = NetlinkReader::new(conn);
    // let links = LinksIterator { iter: reader.into_iter() }.collect();
    let links: Vec<Link>;
    {
        let res: io::Result<Box<LinksIterator<&mut NetlinkConnection>>> = conn.iter_links();
        if res.is_err() {
            return Err(IfConfigError::UnderlyingApiFailed {msg: "Api iter_links() failed".to_string() });
        }
        links = res.unwrap().collect();
    }

    let mut interfaces = Vec::new();

    for link in links {
        let addrs: Vec<Addr> = conn.get_link_addrs(None, &link).map(|i| i.collect()).unwrap_or(Vec::new());
        let ip_addrs: Vec<IpAddrSet> = addrs.iter().map(|a| (a.get_local_ip(), a.get_prefix_len()))
            .filter(|i| i.0.is_some()).map(|i| IpAddrSet {
                unicast_addr: i.0.unwrap(),
                prefix_len: i.1 })
                .collect();

        let interface = Interface::from(link, ip_addrs);
        if interface.is_err() {
            return Err(IfConfigError::UnderlyingApiFailed { msg: "Error creating interface".to_string() });
        }

        interfaces.push(interface.unwrap());
    }

    Ok(interfaces.into_iter())
}

pub struct InterfaceIterator {
    interfaces: IntoIter<Interface>
}

impl Iterator for InterfaceIterator {

    type Item = Interface;
    fn next(&mut self) -> Option<Interface> {
        self.interfaces.next()
    }
}

struct IpAddrSetIterator {
    addrs: IntoIter<IpAddrSet>
}

impl Iterator for IpAddrSetIterator {
    type Item = IpAddrSet;
    fn next(&mut self) -> Option<IpAddrSet> {
        self.addrs.next()
    }
}

pub fn get_interfaces() -> Result<InterfaceIterator, IfConfigError> {
     let conn = NetlinkConnection::new();
     let interfaces = to_interfaces(conn)?;
     Ok(InterfaceIterator { interfaces })
}