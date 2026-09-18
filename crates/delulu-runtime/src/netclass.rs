//! Special-use network addresses (NE-18, owner ruling D-NE-28, PS-0-09).
//!
//! A `--grant net=` value naming loopback, link-local (the `169.254.169.254` cloud-metadata address
//! among them), private, CGNAT, "this network", broadcast, multicast, their IPv6 counterparts, an
//! IPv4-mapped or -compatible IPv6 address, or a name such as `localhost` or
//! `metadata.google.internal`, is REFUSED; it is granted only through the separate explicit spelling
//! `net.special=`. The decision is made on the NORMALIZED form — case, a trailing dot, brackets, a
//! zone id, a port, and every `inet_aton` spelling of IPv4 (`127.1`, `2130706433`, `0x7f.0.0.1`,
//! `0177.0.0.1`) — so no spelling walks past it. Whether a public-looking NAME resolves to such an
//! address is decided at connect time, by the PS-B egress proxy (REMAINING_WORK 4.16).

use std::net::{Ipv4Addr, Ipv6Addr};

/// Why `host` is a special-use address or name, or `None` for an ordinary public host.
pub fn special_use_class(host: &str) -> Option<&'static str> {
    let h = normalize_host(host);
    if let Some(v4) = parse_ipv4_aton(&h) {
        return v4_class(v4);
    }
    if let Ok(v6) = h.parse::<Ipv6Addr>() {
        return v6_class(v6);
    }
    name_class(&h)
}

/// Lowercase; strip `[`…`]`, a `%zone`, a `:port`, and trailing dots.
fn normalize_host(host: &str) -> String {
    let mut h = host.trim().to_ascii_lowercase();
    if let Some(rest) = h.strip_prefix('[') {
        // `[v6]` or `[v6]:port`
        h = rest.split(']').next().unwrap_or(rest).to_string();
    } else if h.matches(':').count() == 1 {
        // `host:port` (a bare IPv6 address has more than one colon)
        h = h.split(':').next().unwrap_or(&h).to_string();
    }
    if let Some(i) = h.find('%') {
        h.truncate(i);
    }
    while h.ends_with('.') {
        h.pop();
    }
    h
}

/// `inet_aton`'s grammar, which is what a resolver accepts: one to four parts, each decimal,
/// octal (leading `0`) or hex (`0x`), the last filling every remaining byte.
fn parse_ipv4_aton(s: &str) -> Option<Ipv4Addr> {
    if s.is_empty() {
        return None;
    }
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() > 4 {
        return None;
    }
    let mut nums: Vec<u64> = Vec::with_capacity(4);
    for p in &parts {
        let v = if let Some(hex) = p.strip_prefix("0x") {
            if hex.is_empty() {
                0
            } else {
                u64::from_str_radix(hex, 16).ok()?
            }
        } else if p.len() > 1 && p.starts_with('0') {
            u64::from_str_radix(&p[1..], 8).ok()?
        } else {
            if p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            p.parse::<u64>().ok()?
        };
        nums.push(v);
    }
    let last = *nums.last()?;
    let head = &nums[..nums.len() - 1];
    if head.iter().any(|&n| n > 255) {
        return None;
    }
    let tail_bytes = 4 - head.len();
    if last >= 1u64 << (8 * tail_bytes as u32) {
        return None;
    }
    let mut v: u32 = 0;
    for (i, &n) in head.iter().enumerate() {
        v |= (n as u32) << (24 - 8 * i as u32);
    }
    v |= last as u32;
    Some(Ipv4Addr::from(v))
}

fn v4_class(ip: Ipv4Addr) -> Option<&'static str> {
    let [a, b, _, _] = ip.octets();
    if a == 0 {
        Some("\"this network\" (0.0.0.0/8)")
    } else if a == 127 {
        Some("loopback (127.0.0.0/8)")
    } else if a == 169 && b == 254 {
        Some("link-local (169.254.0.0/16, which includes the 169.254.169.254 cloud-metadata endpoint)")
    } else if a == 10 || (a == 172 && (16..=31).contains(&b)) || (a == 192 && b == 168) {
        Some("private (10/8, 172.16/12, 192.168/16)")
    } else if a == 100 && (64..=127).contains(&b) {
        Some("carrier-grade NAT (100.64.0.0/10)")
    } else if ip.is_broadcast() {
        Some("broadcast (255.255.255.255)")
    } else if ip.is_multicast() {
        Some("multicast (224.0.0.0/4)")
    } else {
        None
    }
}

fn v6_class(ip: Ipv6Addr) -> Option<&'static str> {
    let s = ip.segments();
    if ip.is_loopback() {
        Some("loopback (::1)")
    } else if ip.is_unspecified() {
        Some("unspecified (::)")
    } else if s[0] & 0xfe00 == 0xfc00 {
        Some("unique-local (fc00::/7)")
    } else if s[0] & 0xffc0 == 0xfe80 {
        Some("link-local (fe80::/10)")
    } else if s[0] & 0xff00 == 0xff00 {
        Some("multicast (ff00::/8)")
    } else if s[..5] == [0, 0, 0, 0, 0] && (s[5] == 0xffff || s[5] == 0) {
        Some("IPv4-mapped or IPv4-compatible (it reaches an IPv4 address by another spelling)")
    } else {
        None
    }
}

fn name_class(h: &str) -> Option<&'static str> {
    const LOOPBACK: &[&str] = &["localhost", "localhost.localdomain", "ip6-localhost", "ip6-loopback"];
    const METADATA: &[&str] =
        &["metadata.google.internal", "metadata.goog", "metadata", "instance-data", "instance-data.ec2.internal"];
    if LOOPBACK.contains(&h) || h.ends_with(".localhost") {
        Some("a loopback name")
    } else if METADATA.contains(&h) {
        Some("a cloud-metadata name")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_spelling_of_a_special_address_is_caught() {
        for h in [
            "169.254.169.254", "127.0.0.1", "127.1", "2130706433", "0x7f.0.0.1", "0177.0.0.1", "0x7f000001",
            "127.0.0.1.", "LOCALHOST", "localhost.", "api.localhost", "10.0.0.8", "172.16.3.4", "172.31.255.255",
            "192.168.1.1", "100.64.0.1", "0.0.0.0", "0", "255.255.255.255", "224.0.0.1", "[::1]", "::1",
            "[::1]:8080", "fe80::1%eth0", "[fe80::1%25en0]", "fd12::3", "fc00::1", "::ffff:127.0.0.1",
            "[::ffff:169.254.169.254]", "::127.0.0.1", "ff02::1", "::", "metadata.google.internal",
            "Metadata.Google.Internal.", "169.254.169.254:80", "localhost:3000",
        ] {
            assert!(special_use_class(h).is_some(), "`{h}` must be special");
        }
    }

    #[test]
    fn ordinary_hosts_are_not() {
        for h in [
            "example.com", "api.weather.example.com", "8.8.8.8", "1.1.1.1", "172.32.0.1", "100.128.0.1",
            "192.169.0.1", "2606:4700::1111", "*.example.com", "localhost.example.com", "metadata.example.com",
            "11.0.0.1", "0x.example.com",
        ] {
            assert!(special_use_class(h).is_none(), "`{h}` is ordinary: {:?}", special_use_class(h));
        }
    }
}
