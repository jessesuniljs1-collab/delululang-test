//! Special-use network addresses (NE-18, owner ruling D-NE-28, PS-0-09).
//!
//! A `--grant net=` value naming loopback, link-local (the `169.254.169.254` cloud-metadata address
//! among them), private, CGNAT, "this network", broadcast, multicast, their IPv6 counterparts, an
//! IPv4-mapped or -compatible IPv6 address, or a name such as `localhost` or
//! `metadata.google.internal`, is REFUSED; it is granted only through the separate explicit spelling
//! `net.special=`. The decision is made on the NORMALIZED form — case, a trailing dot, brackets, a
//! zone id, a port, and every `inet_aton` spelling of IPv4 (`127.1`, `2130706433`, `0x7f.0.0.1`,
//! `0177.0.0.1`) — so no spelling walks past it. Whether a public-looking NAME resolves to such an
//! address is decided at connect time by the egress client (`egress.rs`, PS-B-02), which classifies
//! every address the name resolves to with [`addr_class`] — the same tables, so the grant-time and
//! connect-time answers cannot drift apart.
//!
//! PS-B-02 widened the tables to the IANA special-purpose registries (RFC 6890 and its successors)
//! rather than the handful of ranges an SSRF write-up usually names, because the egress client is
//! where "reachable from the public internet" finally has to be true: the documentation and
//! benchmarking ranges, `240/4`, `192.0.0.0/24`, IPv6 site-local, discard, Teredo and the
//! documentation prefixes. The two translation prefixes are classified by what they CARRY: a NAT64
//! (`64:ff9b::/96`) or 6to4 (`2002::/16`) address reaches the IPv4 address embedded in it, so it is
//! special exactly when that IPv4 address is — `64:ff9b::a9fe:a9fe` is the metadata endpoint spelled
//! through a translator, and an IPv6-only host reaching a public site through DNS64 is not.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

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

/// Why a RESOLVED address is special-use, or `None` when it is an ordinary public address.
///
/// The connect-time half of [`special_use_class`]: the egress client asks it of every address a
/// granted name resolved to, before any of them is dialled. An IPv4-mapped IPv6 address is judged
/// as the IPv4 address it is — a resolver that answers `::ffff:10.0.0.8` has answered `10.0.0.8`.
pub fn addr_class(ip: IpAddr) -> Option<&'static str> {
    match ip.to_canonical() {
        IpAddr::V4(v4) => v4_class(v4),
        IpAddr::V6(v6) => v6_class(v6),
    }
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
    } else if a >= 240 {
        Some("reserved (240.0.0.0/4)")
    } else if a == 192 && b == 0 && ip.octets()[2] == 0 {
        Some("IETF protocol assignments (192.0.0.0/24)")
    } else if (a == 192 && b == 0 && ip.octets()[2] == 2)
        || (a == 198 && b == 51 && ip.octets()[2] == 100)
        || (a == 203 && b == 0 && ip.octets()[2] == 113)
    {
        Some("documentation (192.0.2.0/24, 198.51.100.0/24, 203.0.113.0/24)")
    } else if a == 198 && (b == 18 || b == 19) {
        Some("benchmarking (198.18.0.0/15)")
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
    } else if s[0] & 0xffc0 == 0xfec0 {
        Some("site-local (fec0::/10, deprecated but still routed by some networks)")
    } else if s[..4] == [0x0100, 0, 0, 0] {
        Some("discard-only (100::/64)")
    } else if s[0] == 0x2001 && s[1] == 0 {
        Some("Teredo (2001::/32, which tunnels to an IPv4 address)")
    } else if (s[0] == 0x2001 && s[1] == 0x0db8) || (s[0] == 0x3fff && s[1] & 0xf000 == 0) {
        Some("documentation (2001:db8::/32, 3fff::/20)")
    } else if s[0] == 0x2001 && s[1] == 0x0002 && s[2] == 0 {
        Some("benchmarking (2001:2::/48)")
    } else if s[0] == 0x5f00 {
        Some("SRv6 segment identifiers (5f00::/16)")
    } else if s[..3] == [0x0064, 0xff9b, 0x0001] {
        Some("local-use NAT64 (64:ff9b:1::/48)")
    } else if s[..6] == [0x0064, 0xff9b, 0, 0, 0, 0] {
        // NAT64 (RFC 6052): the last 32 bits ARE an IPv4 address, and a translator will reach it.
        let v4 = Ipv4Addr::new((s[6] >> 8) as u8, s[6] as u8, (s[7] >> 8) as u8, s[7] as u8);
        v4_class(v4).map(|_| "NAT64 (64:ff9b::/96) carrying a special-use IPv4 address")
    } else if s[0] == 0x2002 {
        // 6to4 (RFC 3056): bits 16..48 are the IPv4 address the relay delivers to.
        let v4 = Ipv4Addr::new((s[1] >> 8) as u8, s[1] as u8, (s[2] >> 8) as u8, s[2] as u8);
        v4_class(v4).map(|_| "6to4 (2002::/16) carrying a special-use IPv4 address")
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
            // The translation prefixes are judged by what they carry: these reach PUBLIC addresses.
            "64:ff9b::808:808", "2002:808:808::1", "198.20.0.1", "192.0.3.1", "239.255.255.255.example.com",
        ] {
            assert!(special_use_class(h).is_none(), "`{h}` is ordinary: {:?}", special_use_class(h));
        }
    }

    /// PS-B-02: the IANA special-purpose ranges the egress client must never dial on a plain grant.
    #[test]
    fn the_registry_ranges_added_for_the_egress_client_are_special() {
        for h in [
            "192.0.0.1", "192.0.2.10", "198.51.100.7", "203.0.113.200", "198.18.0.1", "198.19.255.255",
            "240.0.0.1", "250.1.2.3", "fec0::1", "100::1", "2001::1", "2001:0:4136:e378::1", "2001:db8::1",
            "3fff::1", "2001:2::5", "5f00::1", "64:ff9b:1::1",
            // The metadata endpoint and loopback, each spelled through a translator.
            "64:ff9b::a9fe:a9fe", "64:ff9b::7f00:1", "2002:a9fe:a9fe::1", "2002:0a00:0001::",
        ] {
            assert!(special_use_class(h).is_some(), "`{h}` must be special");
        }
    }

    /// The connect-time half agrees with the grant-time half on every address both can see, and
    /// judges a mapped address as the IPv4 address it reaches — `::ffff:8.8.8.8` is public, and
    /// `::ffff:10.0.0.8` is private, whatever the resolver's spelling.
    #[test]
    fn a_resolved_address_is_judged_by_the_same_tables() {
        for (ip, special) in [
            ("127.0.0.1", true),
            ("169.254.169.254", true),
            ("10.0.0.8", true),
            ("8.8.8.8", false),
            ("::1", true),
            ("fd00::1", true),
            ("2606:4700::1111", false),
            ("64:ff9b::a9fe:a9fe", true),
            ("64:ff9b::808:808", false),
            ("::ffff:10.0.0.8", true),
            ("::ffff:8.8.8.8", false),
        ] {
            let addr: IpAddr = ip.parse().unwrap();
            assert_eq!(addr_class(addr).is_some(), special, "`{ip}`: {:?}", addr_class(addr));
            if !ip.starts_with("::ffff:") {
                assert_eq!(
                    addr_class(addr).is_some(),
                    special_use_class(ip).is_some(),
                    "grant time and connect time must agree on `{ip}`"
                );
            }
        }
    }
}
