//! Local adapter discovery, including Wi-Fi networks without an internet route.
use std::net::Ipv4Addr;

pub fn local_ipv4_addresses() -> Vec<String> {
    let mut addresses = adapter_addresses();
    if addresses.is_empty() {
        // UDP connect selects an interface without sending any packet.
        if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
            if socket.connect("10.255.255.255:1").is_ok() {
                if let Ok(std::net::SocketAddr::V4(address)) = socket.local_addr() {
                    addresses.push(*address.ip());
                }
            }
        }
    }
    public_addresses(addresses)
}

fn public_addresses(addresses: Vec<Ipv4Addr>) -> Vec<String> {
    let mut addresses: Vec<_> = addresses
        .into_iter()
        .filter(|ip| {
            !ip.is_loopback() && !ip.is_unspecified() && !ip.is_multicast() && !ip.is_broadcast()
        })
        .collect();
    addresses.sort();
    addresses.dedup();
    addresses.into_iter().map(|ip| ip.to_string()).collect()
}

#[cfg(not(windows))]
fn adapter_addresses() -> Vec<Ipv4Addr> {
    Vec::new()
}

#[cfg(windows)]
fn adapter_addresses() -> Vec<Ipv4Addr> {
    use std::{mem::size_of, ptr};
    use windows_sys::Win32::{
        NetworkManagement::{
            IpHelper::{
                GetAdaptersAddresses, GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_DNS_SERVER,
                GAA_FLAG_SKIP_MULTICAST, IP_ADAPTER_ADDRESSES_LH,
            },
            Ndis::IfOperStatusUp,
        },
        Networking::WinSock::{AF_INET, SOCKADDR_IN},
    };
    let mut bytes = 15_000u32;
    for _ in 0..3 {
        if bytes > 1_048_576 {
            return Vec::new();
        }
        // Struct elements provide the alignment required by the Windows API.
        let count = (bytes as usize).div_ceil(size_of::<IP_ADAPTER_ADDRESSES_LH>());
        let mut buffer = vec![IP_ADAPTER_ADDRESSES_LH::default(); count];
        bytes = (buffer.len() * size_of::<IP_ADAPTER_ADDRESSES_LH>()) as u32;
        // SAFETY: aligned writable buffer with its actual byte capacity. Windows initializes
        // the linked adapter/address structures inside this buffer on success.
        let result = unsafe {
            GetAdaptersAddresses(
                AF_INET as u32,
                GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER,
                ptr::null(),
                buffer.as_mut_ptr(),
                &mut bytes,
            )
        };
        if result == 111 {
            continue;
        } // ERROR_BUFFER_OVERFLOW: adapter configuration changed.
        if result != 0 {
            return Vec::new();
        }
        let mut addresses = Vec::new();
        let mut adapter = buffer.as_ptr();
        // SAFETY: successful GetAdaptersAddresses owns all these linked pointers in buffer,
        // which remains allocated throughout traversal. AF_INET limits socket structures.
        unsafe {
            while let Some(current) = adapter.as_ref() {
                if current.OperStatus == IfOperStatusUp {
                    let mut unicast = current.FirstUnicastAddress;
                    while let Some(address) = unicast.as_ref() {
                        if !address.Address.lpSockaddr.is_null()
                            && address.Address.iSockaddrLength as usize >= size_of::<SOCKADDR_IN>()
                        {
                            let socket = &*address.Address.lpSockaddr.cast::<SOCKADDR_IN>();
                            if socket.sin_family == AF_INET {
                                addresses.push(Ipv4Addr::from(
                                    socket.sin_addr.S_un.S_addr.to_ne_bytes(),
                                ));
                            }
                        }
                        unicast = address.Next;
                    }
                }
                adapter = current.Next;
            }
        }
        return addresses;
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retains_wifi_and_wired_addresses_without_loopback_or_duplicates() {
        let input = [
            "192.0.2.20",
            "192.0.2.10",
            "192.0.2.20",
            "127.0.0.1",
            "0.0.0.0",
            "224.0.0.1",
            "255.255.255.255",
        ];
        assert_eq!(
            public_addresses(input.iter().map(|ip| ip.parse().unwrap()).collect()),
            vec!["192.0.2.10", "192.0.2.20"]
        );
    }
    #[test]
    fn adapter_discovery_returns_valid_non_loopback_ipv4_addresses() {
        for address in local_ipv4_addresses() {
            let ip: Ipv4Addr = address.parse().unwrap();
            assert!(!ip.is_loopback() && !ip.is_unspecified());
        }
    }
}
