//! Network policy for WASI preview-2 (component) guests.
//!
//! A [`NetworkCapability`] is turned into a [`NetworkPolicy`] once, when the instance is created,
//! and the policy answers every socket-address check the guest triggers. Everything is denied
//! unless the capability asks for it. Addresses are compared after normalisation, so an
//! IPv4-mapped IPv6 form (`::ffff:a.b.c.d`) is the same address as `a.b.c.d`: it cannot be used to
//! dodge a rule written for the IPv4 form, and an allowance written for one covers the other.

use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

use crate::error::{Error, Result};
use crate::security::{NetworkCapability, PortRange};

/// What a socket address is being checked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NetUse {
    /// The address of the remote peer: connecting, sending, accepting from, receiving from.
    Remote,
    /// A local address being bound.
    Bind,
    /// A local address being put into listening mode.
    Listen,
}

/// The enforceable form of a [`NetworkCapability`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NetworkPolicy {
    /// No network access at all.
    Deny,
    /// Loopback addresses only.
    Loopback,
    /// Any address, but only these ports.
    Ports(Vec<PortRange>),
    /// Outbound connections to these (normalised) addresses, optionally limited to a port range.
    Hosts(Vec<(IpAddr, Option<PortRange>)>),
    /// Unrestricted.
    Any,
}

fn canonical(ip: IpAddr) -> IpAddr {
    ip.to_canonical()
}

fn in_ranges(ranges: &[PortRange], port: u16) -> bool {
    ranges.iter().any(|r| r.contains(port))
}

impl NetworkPolicy {
    /// Build the policy for a capability, rejecting anything that cannot be enforced.
    ///
    /// Host names are resolved here, once. If the name later resolves to different addresses the
    /// new ones are denied, never allowed.
    pub(crate) fn from_capability(capability: &NetworkCapability) -> Result<Self> {
        match capability {
            NetworkCapability::None => Ok(Self::Deny),
            NetworkCapability::Loopback => Ok(Self::Loopback),
            NetworkCapability::AllowedPorts(ranges) => Ok(Self::Ports(ranges.clone())),
            NetworkCapability::Full => Ok(Self::Any),
            NetworkCapability::AllowedHosts(hosts) => {
                let mut allowed = Vec::new();
                for spec in hosts {
                    if spec.secure {
                        return Err(Error::Capability {
                            message: format!(
                                "HostSpec for '{}' sets `secure`, which cannot be enforced at the \
                                 socket level; restrict the port instead",
                                spec.host
                            ),
                        });
                    }
                    for ip in resolve(&spec.host)? {
                        allowed.push((canonical(ip), spec.ports.clone()));
                    }
                }
                Ok(Self::Hosts(allowed))
            }
        }
    }

    /// Whether sockets of any kind may be created at all.
    pub(crate) fn sockets_enabled(&self) -> bool {
        *self != Self::Deny
    }

    /// Whether the guest may resolve host names. Only unrestricted access allows it: a name lookup
    /// is a request to an outside resolver, and a guest limited to addresses has no need of one.
    pub(crate) fn name_lookup_enabled(&self) -> bool {
        *self == Self::Any
    }

    /// Decide whether the guest may use `addr` for `usage`.
    pub(crate) fn permits(&self, addr: SocketAddr, usage: NetUse) -> bool {
        let ip = canonical(addr.ip());
        let port = addr.port();
        match self {
            Self::Deny => false,
            Self::Any => true,
            Self::Loopback => match usage {
                // An implicit bind to the wildcard address with an ephemeral port is how the OS
                // prepares an outgoing connection; it does not let anything in.
                NetUse::Bind if ip.is_unspecified() && port == 0 => true,
                _ => ip.is_loopback(),
            },
            Self::Ports(ranges) => match usage {
                NetUse::Bind if ip.is_unspecified() && port == 0 => true,
                _ => in_ranges(ranges, port),
            },
            Self::Hosts(hosts) => match usage {
                NetUse::Remote => hosts
                    .iter()
                    .any(|(h, range)| *h == ip && range.as_ref().is_none_or(|r| r.contains(port))),
                NetUse::Bind => ip.is_unspecified() && port == 0,
                // Host rules describe outbound peers only; they never allow listening.
                NetUse::Listen => false,
            },
        }
    }
}

fn resolve(host: &str) -> Result<Vec<IpAddr>> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(vec![ip]);
    }
    let addrs: Vec<IpAddr> = (host, 0u16)
        .to_socket_addrs()
        .map_err(|e| Error::Capability {
            message: format!("cannot resolve allowed host '{host}': {e}"),
        })?
        .map(|a| a.ip())
        .collect();
    if addrs.is_empty() {
        return Err(Error::Capability {
            message: format!("allowed host '{host}' resolved to no addresses"),
        });
    }
    Ok(addrs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::HostSpec;

    fn sa(s: &str) -> SocketAddr {
        s.parse().unwrap()
    }

    fn hosts(ip: &str, ports: Option<(u16, u16)>) -> NetworkPolicy {
        NetworkPolicy::from_capability(&NetworkCapability::AllowedHosts(vec![HostSpec {
            host: ip.to_string(),
            ports: ports.map(|(a, b)| PortRange::new(a, b)),
            secure: false,
        }]))
        .unwrap()
    }

    #[test]
    fn deny_allows_nothing() {
        let p = NetworkPolicy::from_capability(&NetworkCapability::None).unwrap();
        assert!(!p.sockets_enabled());
        for u in [NetUse::Remote, NetUse::Bind, NetUse::Listen] {
            assert!(!p.permits(sa("127.0.0.1:80"), u));
            assert!(!p.permits(sa("0.0.0.0:0"), u));
        }
    }

    #[test]
    fn loopback_allows_loopback_in_both_families_and_mapped_form() {
        let p = NetworkPolicy::Loopback;
        assert!(p.permits(sa("127.0.0.1:80"), NetUse::Remote));
        assert!(p.permits(sa("[::1]:80"), NetUse::Remote));
        assert!(p.permits(sa("[::ffff:127.0.0.1]:80"), NetUse::Remote));
        assert!(!p.permits(sa("8.8.8.8:80"), NetUse::Remote));
        assert!(!p.permits(sa("[::ffff:8.8.8.8]:80"), NetUse::Remote));
        assert!(!p.permits(sa("[2001:db8::1]:80"), NetUse::Remote));
    }

    #[test]
    fn implicit_wildcard_bind_is_allowed_but_listening_on_it_is_not() {
        let p = NetworkPolicy::Loopback;
        assert!(p.permits(sa("0.0.0.0:0"), NetUse::Bind));
        assert!(p.permits(sa("[::]:0"), NetUse::Bind));
        assert!(!p.permits(sa("0.0.0.0:8080"), NetUse::Bind));
        assert!(!p.permits(sa("0.0.0.0:0"), NetUse::Listen));
        assert!(p.permits(sa("127.0.0.1:8080"), NetUse::Listen));
    }

    #[test]
    fn ports_apply_to_any_address_and_both_uses() {
        let p = NetworkPolicy::Ports(vec![PortRange::new(8000, 8010)]);
        assert!(p.permits(sa("8.8.8.8:8005"), NetUse::Remote));
        assert!(p.permits(sa("[::ffff:8.8.8.8]:8005"), NetUse::Remote));
        assert!(!p.permits(sa("8.8.8.8:80"), NetUse::Remote));
        assert!(p.permits(sa("0.0.0.0:8001"), NetUse::Listen));
        assert!(!p.permits(sa("0.0.0.0:9001"), NetUse::Listen));
    }

    #[test]
    fn hosts_compare_after_normalising_mapped_addresses() {
        // Allowance written as IPv4, candidate arrives as IPv4-mapped IPv6 (and the reverse).
        let v4 = hosts("10.0.0.1", None);
        assert!(v4.permits(sa("10.0.0.1:443"), NetUse::Remote));
        assert!(v4.permits(sa("[::ffff:10.0.0.1]:443"), NetUse::Remote));
        assert!(!v4.permits(sa("[::ffff:10.0.0.2]:443"), NetUse::Remote));
        assert!(!v4.permits(sa("10.0.0.2:443"), NetUse::Remote));

        let mapped = hosts("::ffff:10.0.0.1", None);
        assert!(mapped.permits(sa("10.0.0.1:443"), NetUse::Remote));
        assert!(!mapped.permits(sa("10.0.0.2:443"), NetUse::Remote));
    }

    #[test]
    fn hosts_respect_port_ranges_and_never_listen() {
        let p = hosts("10.0.0.1", Some((443, 443)));
        assert!(p.permits(sa("10.0.0.1:443"), NetUse::Remote));
        assert!(!p.permits(sa("10.0.0.1:80"), NetUse::Remote));
        assert!(!p.permits(sa("10.0.0.1:443"), NetUse::Listen));
        assert!(!p.permits(sa("10.0.0.1:443"), NetUse::Bind));
        assert!(p.permits(sa("0.0.0.0:0"), NetUse::Bind));
    }

    #[test]
    fn name_lookup_only_for_unrestricted() {
        assert!(NetworkPolicy::Any.name_lookup_enabled());
        for p in [
            NetworkPolicy::Deny,
            NetworkPolicy::Loopback,
            hosts("10.0.0.1", None),
        ] {
            assert!(!p.name_lookup_enabled());
        }
    }

    #[test]
    fn secure_and_unresolvable_hosts_are_errors() {
        let secure = NetworkCapability::AllowedHosts(vec![HostSpec {
            host: "10.0.0.1".to_string(),
            ports: None,
            secure: true,
        }]);
        assert!(matches!(
            NetworkPolicy::from_capability(&secure),
            Err(Error::Capability { .. })
        ));
        let missing = NetworkCapability::AllowedHosts(vec![HostSpec {
            host: "no-such-host.invalid".to_string(),
            ports: None,
            secure: false,
        }]);
        assert!(matches!(
            NetworkPolicy::from_capability(&missing),
            Err(Error::Capability { .. })
        ));
    }
}
