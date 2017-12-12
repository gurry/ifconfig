// TODO: Eventually remove dependency on crate pnetlink and replace it with some netlink code written right here.
// Alternatively, maybe write a properly design netlink crate as well and depend on that.
// See Go's package https://github.com/vishvananda/netlink for inspiration
extern crate libc;
extern crate pnetlink;

use pnetlink::packet::netlink::NetlinkConnection;
use pnetlink::packet::route::link::{Links,Link};
use pnetlink::packet::route::addr::{Addresses,Addr};

// pub struct Interface(PIP_ADAPTER_ADDRESSES);

// impl Interface {
//     pub fn index(&self) -> u32 {
//         let mut index = unsafe {(*self.0).__bindgen_anon_1.__bindgen_anon_1.IfIndex }; // Using ipV4 if index
//         if index == 0 { // If ipv4 index was zero. As per MSDN this can happen if ipv4 is disabled on this interface
//             index = unsafe { (*self.0).Ipv6IfIndex };
//         }

//         index
//     }

//     pub fn mtu(&self) -> u32 { // TODO: what does a value of 0xffffffff mean for the MTU field in the inner struct
//         unsafe { (*self.0).Mtu }
//     }

//     pub fn name(&self) -> &str {
//         unsafe { CStr::from_ptr((*self.0).AdapterName) }.to_str().expect("AdapterName could not be converted to &str")
//     }

//     // TODO: can we return OsStr here instead?
//     pub fn friendly_name(&self) -> String {
//         // Must return an owned string here because there's no way to return a zero-copy &str type since Windows wide strings very different from Rust's utf8 strings
//         unsafe { WideCStr::from_ptr_str((*self.0).FriendlyName) }.to_string().expect("FriendlyName could not be converted to String")
//     }

//     // TODO: can we return OsStr here instead?
//     pub fn description(&self) -> String {
//         // Must return an owned string here because there's no way to return a zero-copy &str type since Windows wide strings very different from Rust's utf8 strings
//         unsafe { WideCStr::from_ptr_str((*self.0).Description) }.to_string().expect("Description could not be converted to String")
//     }

//     pub fn hw_addr(&self) -> Result<Option<HardwareAddr>, IfConfigError> {
//         unsafe {
//             let len = (*self.0).PhysicalAddressLength as usize;
//             if len == 0 {
//                 Ok(None)
//             }
//             else if len != EUI48_LEN {
//                  Err(IfConfigError::BadHardwareAddr { len })
//             }
//             else {
//                 let ptr: *const u8 = (*self.0).PhysicalAddress.as_ptr();
//                 let bytes =  &*(ptr as *const [u8; 6]);
//                 Ok(Some(HardwareAddr::from_bytes(bytes)))
//             }
//          }
//     }

//     // TODO: this should also include anycast addresses they way golang implementatio does
//     /// Get the adapter's ip addresses (unicast ip addresses)
//     pub fn ip_addrs(&self) -> impl Iterator<Item=IpAddr> { // TODO: Should we rename this to unicast_ip_addresses?
//         IpAddrIterator::from(unsafe { (*self.0).FirstUnicastAddress })
//     }

//     // pub fn ip_addrs_multicast(&self) -> impl Iterator<Item=IpAddr> { // TODO: Should we rename this to unicast_ip_addresses?
//     //     IpAddrIterator::from(self.0.FistUnicastAddress)
//     // }


//     pub fn flags(&self) -> Flags {
//         // Shamelessly copied from what the Golang people are doing.
//         // There is also a comment that ideally the below info should come from MIB_IF_ROW2.AccessType. But go with this for now.
//         unsafe {
//             match (*self.0).IfType {
//                 IF_TYPE_ETHERNET_CSMACD | IF_TYPE_ISO88025_TOKENRING | IF_TYPE_IEEE80211 | IF_TYPE_IEEE1394 => {
//                     Flags::BROADCAST | Flags::MULTICAST
//                 },
//                 IF_TYPE_PPP | IF_TYPE_TUNNEL => {
//                     Flags::POINT_TO_POINT | Flags::MULTICAST
//                 },
//                 IF_TYPE_SOFTWARE_LOOPBACK => {
//                     Flags::LOOPBACK | Flags::MULTICAST
//                 },
//                 IF_TYPE_ATM => {
//                     Flags::BROADCAST | Flags::POINT_TO_POINT | Flags::MULTICAST // assume all services available; LANE, point-to-point and point-to-multipoint
//                 }
//                 _ => Flags::empty()
//             }
//         }
//     }
// }

// pub struct InterfaceIterator {
//     _adapter_addresses_buffer: Vec<u8>,
//     current_ptr: PIP_ADAPTER_ADDRESSES,
// }

// impl InterfaceIterator {
//     fn from(mut adapter_addresses_buffer: Vec<u8>) -> Self {
//         let start_ptr: PIP_ADAPTER_ADDRESSES = unsafe { std::mem::transmute(adapter_addresses_buffer.as_mut_ptr()) };
//         Self {_adapter_addresses_buffer: adapter_addresses_buffer, current_ptr: start_ptr }
//     }
// }

// impl Iterator for InterfaceIterator {
//     type Item = Interface;
//     fn next(&mut self) -> Option<Interface> {
//         if self.current_ptr != std::ptr::null_mut() {
//             let interface = Interface(self.current_ptr);
//             self.current_ptr = unsafe { (*self.current_ptr).Next } ;
//             Some(interface)
//         }
//         else {
//             None
//         }
//     }
// }

// struct IpAddrIterator {
//     _adapter_unicast_ptr: PIP_ADAPTER_UNICAST_ADDRESS_LH,
//     current_ptr: PIP_ADAPTER_UNICAST_ADDRESS_LH,
// }

// impl IpAddrIterator {
//     fn from(adapter_unicast_ptr: PIP_ADAPTER_UNICAST_ADDRESS_LH) -> Self {
//         Self { _adapter_unicast_ptr: adapter_unicast_ptr, current_ptr: adapter_unicast_ptr }
//     }
// }

// impl Iterator for IpAddrIterator {
//     type Item = IpAddr;
//     fn next(&mut self) -> Option<IpAddr> {
//         if self.current_ptr != std::ptr::null_mut() {
//             let ip_addr = unsafe { socket_address_to_ipaddr(&(*self.current_ptr).Address) };
//             self.current_ptr = unsafe { (*self.current_ptr).Next };
//             Some(ip_addr)
//         }
//         else {
//             None
//         }
//     }
// }
pub fn get_interfaces() -> Result<impl Iterator<Item=Interface>, Error> {
    unimplemented!()
    // let mut conn = NetlinkConnection::new();
    // let links = conn.iter_links().unwrap().collect::<Vec<_>>();
}