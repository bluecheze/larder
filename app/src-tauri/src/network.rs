use std::net::IpAddr;

/// IPv4 addresses other machines on your network can use to reach this
/// one. Skips loopback and link-local (169.254.x.x), which nobody else
/// can route to anyway.
pub fn lan_addresses() -> Vec<String> {
    let mut found: Vec<String> = if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|iface| match iface.ip() {
            IpAddr::V4(ip) if !ip.is_loopback() && !ip.is_link_local() => Some(ip.to_string()),
            _ => None,
        })
        .collect();
    // Home routers almost always hand out 192.168.x.x, so those go first.
    // Docker and VPN adapters tend to live in 10.x and 172.x.
    found.sort_by_key(|ip| (!ip.starts_with("192.168."), ip.clone()));
    found.dedup();
    found
}
