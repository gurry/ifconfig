mod bindings;

extern crate winapi;
use self::winapi::{AF_UNSPEC, ERROR_SUCCESS, ERROR_BUFFER_OVERFLOW, ULONG};

//fn GetAdaptersAddresses ( Family : ULONG , Flags : ULONG , Reserved : PVOID , AdapterAddresses : PIP_ADAPTER_ADDRESSES , SizePointer : PULONG , ) -> ULONG; 

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
}   extern crate winapi;
use self::winapi::{AF_UNSPEC, ERROR_SUCCESS, ERROR_BUFFER_OVERFLOW, ULONG};

//fn GetAdaptersAddresses ( Family : ULONG , Flags : ULONG , Reserved : PVOID , AdapterAddresses : PIP_ADAPTER_ADDRESSES , SizePointer : PULONG , ) -> ULONG; 

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