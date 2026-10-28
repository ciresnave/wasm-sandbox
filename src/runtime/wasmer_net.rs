//! Network policy for the Wasmer backend: a wasix [`VirtualNetworking`] that checks every socket
//! address the guest uses against a [`NetworkPolicy`] before handing the call to a real network.
//!
//! wasix never opens a socket itself, so wrapping this one trait covers every socket a guest can
//! make. The rules are the same as for components (`runtime::network_policy`), applied per call:
//! connecting, binding, listening, accepting, and each UDP datagram sent or received. Everything
//! the policy does not name is denied: raw and ICMP sockets, interface and routing changes,
//! multicast and broadcast, and name lookups unless the policy is unrestricted.
//!
//! Every method of [`VirtualNetworking`] has an `Unsupported` default, so a method added upstream
//! that this wrapper does not override is refused rather than passed through.

use std::fmt;
use std::mem::MaybeUninit;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use wasmer_wasix::virtual_net::{
    InterestHandler, IpCidr, IpRoute, NetworkError, Result, SocketStatus, StreamSecurity,
    VirtualConnectionlessSocket, VirtualIcmpSocket, VirtualIoSource, VirtualNetworking,
    VirtualRawSocket, VirtualSocket, VirtualTcpBoundSocket, VirtualTcpListener, VirtualTcpSocket,
    VirtualUdpSocket,
};

use crate::runtime::network_policy::{NetUse, NetworkPolicy};

/// A [`VirtualNetworking`] that only lets the guest do what its [`NetworkPolicy`] permits.
#[derive(Debug)]
pub(crate) struct PolicyNetworking {
    policy: Arc<NetworkPolicy>,
    inner: Arc<dyn VirtualNetworking>,
}

impl PolicyNetworking {
    pub(crate) fn new(policy: NetworkPolicy, inner: Arc<dyn VirtualNetworking>) -> Self {
        Self {
            policy: Arc::new(policy),
            inner,
        }
    }

    fn check(&self, addr: SocketAddr, usage: NetUse) -> Result<()> {
        if self.policy.permits(addr, usage) {
            Ok(())
        } else {
            tracing::debug!(%addr, ?usage, "guest network access denied by policy");
            Err(NetworkError::PermissionDenied)
        }
    }
}

fn denied<T>() -> Result<T> {
    Err(NetworkError::PermissionDenied)
}

#[async_trait::async_trait]
impl VirtualNetworking for PolicyNetworking {
    // Configuring the host's virtual network is never part of a guest's traffic.
    async fn bridge(&self, _: &str, _: &str, _: StreamSecurity) -> Result<()> {
        denied()
    }

    async fn unbridge(&self) -> Result<()> {
        denied()
    }

    async fn dhcp_acquire(&self) -> Result<Vec<IpAddr>> {
        denied()
    }

    async fn ip_add(&self, _: IpAddr, _: u8) -> Result<()> {
        denied()
    }

    async fn ip_remove(&self, _: IpAddr) -> Result<()> {
        denied()
    }

    async fn ip_clear(&self) -> Result<()> {
        denied()
    }

    async fn ip_list(&self) -> Result<Vec<IpCidr>> {
        denied()
    }

    async fn mac(&self) -> Result<[u8; 6]> {
        denied()
    }

    async fn gateway_set(&self, _: IpAddr) -> Result<()> {
        denied()
    }

    async fn route_add(
        &self,
        _: IpCidr,
        _: IpAddr,
        _: Option<Duration>,
        _: Option<Duration>,
    ) -> Result<()> {
        denied()
    }

    async fn route_remove(&self, _: IpAddr) -> Result<()> {
        denied()
    }

    async fn route_clear(&self) -> Result<()> {
        denied()
    }

    async fn route_list(&self) -> Result<Vec<IpRoute>> {
        denied()
    }

    // Raw and ICMP sockets are not part of any `NetworkCapability`.
    async fn bind_raw(&self) -> Result<Box<dyn VirtualRawSocket + Sync>> {
        denied()
    }

    async fn bind_icmp(&self, _: IpAddr) -> Result<Box<dyn VirtualIcmpSocket + Sync>> {
        denied()
    }

    async fn listen_tcp(
        &self,
        addr: SocketAddr,
        only_v6: bool,
        reuse_port: bool,
        reuse_addr: bool,
    ) -> Result<Box<dyn VirtualTcpListener + Sync>> {
        self.check(addr, NetUse::Listen)?;
        let listener = self
            .inner
            .listen_tcp(addr, only_v6, reuse_port, reuse_addr)
            .await?;
        Ok(Box::new(PolicyListener {
            inner: listener,
            policy: self.policy.clone(),
        }))
    }

    async fn bind_tcp(
        &self,
        addr: SocketAddr,
        only_v6: bool,
        reuse_port: bool,
        reuse_addr: bool,
    ) -> Result<Box<dyn VirtualTcpBoundSocket + Sync>> {
        self.check(addr, NetUse::Bind)?;
        let socket = self
            .inner
            .bind_tcp(addr, only_v6, reuse_port, reuse_addr)
            .await?;
        Ok(Box::new(PolicyBoundSocket {
            inner: socket,
            policy: self.policy.clone(),
        }))
    }

    async fn bind_udp(
        &self,
        addr: SocketAddr,
        reuse_port: bool,
        reuse_addr: bool,
    ) -> Result<Box<dyn VirtualUdpSocket + Sync>> {
        self.check(addr, NetUse::Bind)?;
        let socket = self.inner.bind_udp(addr, reuse_port, reuse_addr).await?;
        Ok(Box::new(PolicyUdpSocket {
            inner: socket,
            policy: self.policy.clone(),
        }))
    }

    async fn connect_tcp(
        &self,
        addr: SocketAddr,
        peer: SocketAddr,
    ) -> Result<Box<dyn VirtualTcpSocket + Sync>> {
        self.check(addr, NetUse::Bind)?;
        self.check(peer, NetUse::Remote)?;
        self.inner.connect_tcp(addr, peer).await
    }

    async fn resolve(
        &self,
        host: &str,
        port: Option<u16>,
        dns_server: Option<IpAddr>,
    ) -> Result<Vec<IpAddr>> {
        if !self.policy.name_lookup_enabled() {
            return denied();
        }
        self.inner.resolve(host, port, dns_server).await
    }
}

/// A listening socket that closes connections from peers the policy does not allow.
struct PolicyListener {
    inner: Box<dyn VirtualTcpListener + Sync>,
    policy: Arc<NetworkPolicy>,
}

impl fmt::Debug for PolicyListener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PolicyListener")
            .field("inner", &self.inner)
            .finish()
    }
}

impl VirtualIoSource for PolicyListener {
    fn remove_handler(&mut self) {
        self.inner.remove_handler();
    }

    fn poll_read_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<usize>> {
        self.inner.poll_read_ready(cx)
    }

    fn poll_write_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<usize>> {
        self.inner.poll_write_ready(cx)
    }
}

impl VirtualTcpListener for PolicyListener {
    fn try_accept(&mut self) -> Result<(Box<dyn VirtualTcpSocket + Sync>, SocketAddr)> {
        loop {
            let (socket, peer) = self.inner.try_accept()?;
            if self.policy.permits(peer, NetUse::Remote) {
                return Ok((socket, peer));
            }
            tracing::debug!(%peer, "connection from a peer the policy does not allow was closed");
            drop(socket);
        }
    }

    fn set_handler(&mut self, handler: Box<dyn InterestHandler + Send + Sync>) -> Result<()> {
        self.inner.set_handler(handler)
    }

    fn addr_local(&self) -> Result<SocketAddr> {
        self.inner.addr_local()
    }

    fn set_ttl(&mut self, ttl: u8) -> Result<()> {
        self.inner.set_ttl(ttl)
    }

    fn ttl(&self) -> Result<u8> {
        self.inner.ttl()
    }
}

/// A bound, not yet listening or connected TCP socket.
struct PolicyBoundSocket {
    inner: Box<dyn VirtualTcpBoundSocket + Sync>,
    policy: Arc<NetworkPolicy>,
}

impl fmt::Debug for PolicyBoundSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PolicyBoundSocket")
            .field("inner", &self.inner)
            .finish()
    }
}

impl VirtualTcpBoundSocket for PolicyBoundSocket {
    fn addr_local(&self) -> Result<SocketAddr> {
        self.inner.addr_local()
    }

    fn listen(&mut self) -> Result<Box<dyn VirtualTcpListener + Sync>> {
        let local = self.inner.addr_local()?;
        if !self.policy.permits(local, NetUse::Listen) {
            return denied();
        }
        let listener = self.inner.listen()?;
        Ok(Box::new(PolicyListener {
            inner: listener,
            policy: self.policy.clone(),
        }))
    }

    fn connect(&mut self, peer: SocketAddr) -> Result<Box<dyn VirtualTcpSocket + Sync>> {
        if !self.policy.permits(peer, NetUse::Remote) {
            return denied();
        }
        self.inner.connect(peer)
    }

    fn set_ttl(&mut self, ttl: u32) -> Result<()> {
        self.inner.set_ttl(ttl)
    }

    fn ttl(&self) -> Result<u32> {
        self.inner.ttl()
    }
}

/// A UDP socket that checks the peer of every datagram. A datagram from a peer the policy does not
/// allow is dropped; sending to one is an error.
struct PolicyUdpSocket {
    inner: Box<dyn VirtualUdpSocket + Sync>,
    policy: Arc<NetworkPolicy>,
}

impl fmt::Debug for PolicyUdpSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PolicyUdpSocket")
            .field("inner", &self.inner)
            .finish()
    }
}

impl VirtualIoSource for PolicyUdpSocket {
    fn remove_handler(&mut self) {
        self.inner.remove_handler();
    }

    fn poll_read_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<usize>> {
        self.inner.poll_read_ready(cx)
    }

    fn poll_write_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<usize>> {
        self.inner.poll_write_ready(cx)
    }
}

impl VirtualSocket for PolicyUdpSocket {
    fn set_ttl(&mut self, ttl: u32) -> Result<()> {
        self.inner.set_ttl(ttl)
    }

    fn ttl(&self) -> Result<u32> {
        self.inner.ttl()
    }

    fn addr_local(&self) -> Result<SocketAddr> {
        self.inner.addr_local()
    }

    fn status(&self) -> Result<SocketStatus> {
        self.inner.status()
    }

    fn last_error(&self) -> Result<Option<NetworkError>> {
        self.inner.last_error()
    }

    fn set_handler(&mut self, handler: Box<dyn InterestHandler + Send + Sync>) -> Result<()> {
        self.inner.set_handler(handler)
    }
}

impl VirtualConnectionlessSocket for PolicyUdpSocket {
    fn try_send_to(&mut self, data: &[u8], addr: SocketAddr) -> Result<usize> {
        if !self.policy.permits(addr, NetUse::Remote) {
            return denied();
        }
        self.inner.try_send_to(data, addr)
    }

    fn try_recv_from(
        &mut self,
        buf: &mut [MaybeUninit<u8>],
        peek: bool,
    ) -> Result<(usize, SocketAddr)> {
        loop {
            let (len, peer) = self.inner.try_recv_from(buf, peek)?;
            if self.policy.permits(peer, NetUse::Remote) {
                return Ok((len, peer));
            }
            tracing::debug!(%peer, "datagram from a peer the policy does not allow was dropped");
            if peek {
                // A peeked datagram stays queued; consume it or this would loop on it forever.
                self.inner.try_recv_from(buf, false)?;
            }
        }
    }
}

impl VirtualUdpSocket for PolicyUdpSocket {
    fn set_broadcast(&mut self, broadcast: bool) -> Result<()> {
        if broadcast && !self.policy.name_lookup_enabled() {
            return denied();
        }
        self.inner.set_broadcast(broadcast)
    }

    fn broadcast(&self) -> Result<bool> {
        self.inner.broadcast()
    }

    fn set_multicast_loop_v4(&mut self, val: bool) -> Result<()> {
        self.inner.set_multicast_loop_v4(val)
    }

    fn multicast_loop_v4(&self) -> Result<bool> {
        self.inner.multicast_loop_v4()
    }

    fn set_multicast_loop_v6(&mut self, val: bool) -> Result<()> {
        self.inner.set_multicast_loop_v6(val)
    }

    fn multicast_loop_v6(&self) -> Result<bool> {
        self.inner.multicast_loop_v6()
    }

    fn set_multicast_ttl_v4(&mut self, ttl: u32) -> Result<()> {
        self.inner.set_multicast_ttl_v4(ttl)
    }

    fn multicast_ttl_v4(&self) -> Result<u32> {
        self.inner.multicast_ttl_v4()
    }

    fn join_multicast_v4(&mut self, multiaddr: Ipv4Addr, iface: Ipv4Addr) -> Result<()> {
        if !self.policy.name_lookup_enabled() {
            return denied();
        }
        self.inner.join_multicast_v4(multiaddr, iface)
    }

    fn leave_multicast_v4(&mut self, multiaddr: Ipv4Addr, iface: Ipv4Addr) -> Result<()> {
        self.inner.leave_multicast_v4(multiaddr, iface)
    }

    fn join_multicast_v6(&mut self, multiaddr: Ipv6Addr, iface: u32) -> Result<()> {
        if !self.policy.name_lookup_enabled() {
            return denied();
        }
        self.inner.join_multicast_v6(multiaddr, iface)
    }

    fn leave_multicast_v6(&mut self, multiaddr: Ipv6Addr, iface: u32) -> Result<()> {
        self.inner.leave_multicast_v6(multiaddr, iface)
    }

    fn addr_peer(&self) -> Result<Option<SocketAddr>> {
        self.inner.addr_peer()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::{NetworkCapability, PortRange};
    use std::net::{TcpListener, TcpStream, UdpSocket};
    use wasmer_wasix::virtual_net::UnsupportedVirtualNetworking;
    use wasmer_wasix::virtual_net::host::LocalNetworking;

    fn policy(capability: NetworkCapability) -> NetworkPolicy {
        NetworkPolicy::from_capability(&capability).unwrap()
    }

    fn wrap(capability: NetworkCapability) -> PolicyNetworking {
        PolicyNetworking::new(policy(capability), Arc::new(LocalNetworking::new()))
    }

    fn sa(s: &str) -> SocketAddr {
        s.parse().unwrap()
    }

    /// Stands in for a real network and answers every method it implements with `InvalidFd`, so a
    /// call that reaches it is distinguishable from the `Unsupported` of an un-overridden default
    /// and from the `PermissionDenied` of a refusal.
    #[derive(Debug)]
    struct Sentinel;

    #[async_trait::async_trait]
    impl VirtualNetworking for Sentinel {
        async fn listen_tcp(
            &self,
            _: SocketAddr,
            _: bool,
            _: bool,
            _: bool,
        ) -> Result<Box<dyn VirtualTcpListener + Sync>> {
            Err(NetworkError::InvalidFd)
        }
        async fn bind_tcp(
            &self,
            _: SocketAddr,
            _: bool,
            _: bool,
            _: bool,
        ) -> Result<Box<dyn VirtualTcpBoundSocket + Sync>> {
            Err(NetworkError::InvalidFd)
        }
        async fn bind_udp(
            &self,
            _: SocketAddr,
            _: bool,
            _: bool,
        ) -> Result<Box<dyn VirtualUdpSocket + Sync>> {
            Err(NetworkError::InvalidFd)
        }
        async fn connect_tcp(
            &self,
            _: SocketAddr,
            _: SocketAddr,
        ) -> Result<Box<dyn VirtualTcpSocket + Sync>> {
            Err(NetworkError::InvalidFd)
        }
        async fn resolve(&self, _: &str, _: Option<u16>, _: Option<IpAddr>) -> Result<Vec<IpAddr>> {
            Err(NetworkError::InvalidFd)
        }
    }

    macro_rules! err_of {
        ($e:expr) => {
            match $e {
                Err(e) => e,
                Ok(_) => panic!("expected an error from {}", stringify!($e)),
            }
        };
    }

    /// Every method is overridden: with nothing allowed each answers `PermissionDenied`, where an
    /// un-overridden method would answer the trait default, `Unsupported`.
    #[tokio::test]
    async fn every_method_is_overridden_and_denies_when_nothing_is_allowed() {
        let net = PolicyNetworking::new(NetworkPolicy::Deny, Arc::new(Sentinel));
        let a = sa("127.0.0.1:80");
        let ip = a.ip();
        let denied = NetworkError::PermissionDenied;
        assert_eq!(
            err_of!(net.bridge("n", "t", StreamSecurity::Unencrypted).await),
            denied
        );
        assert_eq!(err_of!(net.unbridge().await), denied);
        assert_eq!(err_of!(net.dhcp_acquire().await), denied);
        assert_eq!(err_of!(net.ip_add(ip, 8).await), denied);
        assert_eq!(err_of!(net.ip_remove(ip).await), denied);
        assert_eq!(err_of!(net.ip_clear().await), denied);
        assert_eq!(err_of!(net.ip_list().await), denied);
        assert_eq!(err_of!(net.mac().await), denied);
        assert_eq!(err_of!(net.gateway_set(ip).await), denied);
        let cidr = IpCidr { ip, prefix: 8 };
        assert_eq!(err_of!(net.route_add(cidr, ip, None, None).await), denied);
        assert_eq!(err_of!(net.route_remove(ip).await), denied);
        assert_eq!(err_of!(net.route_clear().await), denied);
        assert_eq!(err_of!(net.route_list().await), denied);
        assert_eq!(err_of!(net.bind_raw().await), denied);
        assert_eq!(err_of!(net.bind_icmp(ip).await), denied);
        assert_eq!(
            err_of!(net.listen_tcp(a, false, false, false).await),
            denied
        );
        assert_eq!(err_of!(net.bind_tcp(a, false, false, false).await), denied);
        assert_eq!(err_of!(net.bind_udp(a, false, false).await), denied);
        assert_eq!(err_of!(net.connect_tcp(a, a).await), denied);
        assert_eq!(err_of!(net.resolve("localhost", None, None).await), denied);
    }

    /// With everything allowed the methods that carry traffic reach the inner network (`InvalidFd`
    /// from the sentinel), and the ones that configure the host network stay denied.
    #[tokio::test]
    async fn allowed_calls_reach_the_inner_network_and_host_configuration_stays_denied() {
        let net = PolicyNetworking::new(NetworkPolicy::Any, Arc::new(Sentinel));
        let a = sa("10.0.0.1:80");
        let reached = NetworkError::InvalidFd;
        assert_eq!(
            err_of!(net.listen_tcp(a, false, false, false).await),
            reached
        );
        assert_eq!(err_of!(net.bind_tcp(a, false, false, false).await), reached);
        assert_eq!(err_of!(net.bind_udp(a, false, false).await), reached);
        assert_eq!(err_of!(net.connect_tcp(a, a).await), reached);
        assert_eq!(
            err_of!(net.resolve("example.com", None, None).await),
            reached
        );
        let denied = NetworkError::PermissionDenied;
        assert_eq!(err_of!(net.bind_raw().await), denied);
        assert_eq!(err_of!(net.bind_icmp(a.ip()).await), denied);
        assert_eq!(err_of!(net.dhcp_acquire().await), denied);
        assert_eq!(err_of!(net.mac().await), denied);
        // Positive control for the test above: the bare trait default is `Unsupported`.
        assert_eq!(
            err_of!(UnsupportedVirtualNetworking::default().mac().await),
            NetworkError::Unsupported
        );
    }

    #[tokio::test]
    async fn name_lookup_needs_unrestricted_access() {
        for capability in [
            NetworkCapability::Loopback,
            NetworkCapability::AllowedPorts(vec![PortRange::new(1, 65535)]),
        ] {
            let net = PolicyNetworking::new(policy(capability), Arc::new(Sentinel));
            assert_eq!(
                err_of!(net.resolve("localhost", None, None).await),
                NetworkError::PermissionDenied
            );
        }
    }

    #[tokio::test]
    async fn connect_follows_the_policy() {
        let host = TcpListener::bind("127.0.0.1:0").unwrap();
        let peer = host.local_addr().unwrap();
        let any = sa("0.0.0.0:0");

        // Positive control: an unrestricted policy connects to the same listener.
        assert!(
            wrap(NetworkCapability::Full)
                .connect_tcp(any, peer)
                .await
                .is_ok()
        );
        assert!(
            wrap(NetworkCapability::Loopback)
                .connect_tcp(any, peer)
                .await
                .is_ok()
        );
        assert_eq!(
            err_of!(wrap(NetworkCapability::None).connect_tcp(any, peer).await),
            NetworkError::PermissionDenied
        );
        // Outside the port range, and a non-loopback address under `Loopback`.
        let other = PortRange::new(peer.port().wrapping_add(1).max(1), 65535);
        let outside = if other.contains(peer.port()) {
            PortRange::new(1, peer.port() - 1)
        } else {
            other
        };
        assert_eq!(
            err_of!(
                wrap(NetworkCapability::AllowedPorts(vec![outside]))
                    .connect_tcp(any, peer)
                    .await
            ),
            NetworkError::PermissionDenied
        );
        assert_eq!(
            err_of!(
                wrap(NetworkCapability::Loopback)
                    .connect_tcp(any, sa("192.0.2.1:80"))
                    .await
            ),
            NetworkError::PermissionDenied
        );
        // An explicit local port is a bind, which `Loopback` only allows on loopback.
        assert_eq!(
            err_of!(
                wrap(NetworkCapability::Loopback)
                    .connect_tcp(sa("192.0.2.1:5000"), peer)
                    .await
            ),
            NetworkError::PermissionDenied
        );
    }

    #[tokio::test]
    async fn an_ipv4_mapped_peer_is_the_same_address() {
        let host = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = host.local_addr().unwrap().port();
        let mapped = sa(&format!("[::ffff:127.0.0.1]:{port}"));
        let net = wrap(NetworkCapability::Loopback);
        assert!(net.check(mapped, NetUse::Remote).is_ok());
        let foreign = sa(&format!("[::ffff:192.0.2.1]:{port}"));
        assert!(net.check(foreign, NetUse::Remote).is_err());
    }

    #[tokio::test]
    async fn listen_is_denied_unless_the_capability_allows_it() {
        let free = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap();
        for capability in [
            NetworkCapability::None,
            NetworkCapability::AllowedHosts(vec![crate::security::HostSpec {
                host: "127.0.0.1".into(),
                ports: None,
                secure: false,
            }]),
        ] {
            let net = wrap(capability);
            assert_eq!(
                err_of!(net.listen_tcp(free, false, false, false).await),
                NetworkError::PermissionDenied
            );
            assert_eq!(
                err_of!(net.bind_tcp(free, false, false, false).await),
                NetworkError::PermissionDenied
            );
        }
        // Positive control.
        assert!(
            wrap(NetworkCapability::Loopback)
                .listen_tcp(free, false, false, false)
                .await
                .is_ok()
        );
    }

    /// A connection from a peer the policy does not allow is closed and never surfaces.
    #[tokio::test]
    async fn accept_closes_connections_from_disallowed_peers() {
        // A free port, then a policy that lists exactly that port: the client's ephemeral source
        // port is outside it, so its connection is from a disallowed peer.
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let addr = sa(&format!("127.0.0.1:{port}"));
        let net = wrap(NetworkCapability::AllowedPorts(vec![PortRange::new(
            port, port,
        )]));
        let mut listener = net.listen_tcp(addr, false, false, false).await.unwrap();

        let mut client = TcpStream::connect(addr).unwrap();
        client.set_nonblocking(true).unwrap();
        let mut closed = false;
        for _ in 0..100 {
            assert_eq!(
                err_of!(listener.try_accept()),
                NetworkError::WouldBlock,
                "a disallowed peer must never be returned"
            );
            let mut byte = [0u8; 1];
            match std::io::Read::read(&mut client, &mut byte) {
                Ok(0) => {
                    closed = true;
                    break;
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => {
                    closed = true;
                    break;
                }
                Ok(_) => panic!("unexpected data"),
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(closed, "the server side of the connection must be closed");

        // Positive control: under `Loopback` the same flow returns the connection.
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let addr = sa(&format!("127.0.0.1:{port}"));
        let net = wrap(NetworkCapability::Loopback);
        let mut listener = net.listen_tcp(addr, false, false, false).await.unwrap();
        let _client = TcpStream::connect(addr).unwrap();
        let mut accepted = false;
        for _ in 0..100 {
            if listener.try_accept().is_ok() {
                accepted = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(accepted);
    }

    #[tokio::test]
    async fn udp_datagrams_from_disallowed_sources_are_dropped_and_sends_are_checked() {
        let net = wrap(NetworkCapability::AllowedHosts(vec![
            crate::security::HostSpec {
                host: "127.0.0.1".into(),
                ports: None,
                secure: false,
            },
        ]));
        // Only the wildcard address with an ephemeral port may be bound under a host rule.
        let mut socket = net.bind_udp(sa("0.0.0.0:0"), false, false).await.unwrap();
        let port = VirtualSocket::addr_local(socket.as_ref()).unwrap().port();
        let target = sa(&format!("127.0.0.1:{port}"));

        // 127.0.0.2 is loopback but not the allowed host; it sends first, so if it were not
        // dropped it would be received first.
        let bad = UdpSocket::bind("127.0.0.2:0").unwrap();
        bad.send_to(b"bad", target).unwrap();
        let good = UdpSocket::bind("127.0.0.1:0").unwrap();
        good.send_to(b"good", target).unwrap();

        let mut buf = [MaybeUninit::<u8>::uninit(); 16];
        let mut received = None;
        for _ in 0..100 {
            match socket.try_recv_from(&mut buf, true) {
                Ok((len, from)) => {
                    // `peek` leaves an allowed datagram queued; read it for real.
                    let (len2, from2) = socket.try_recv_from(&mut buf, false).unwrap();
                    assert_eq!((len, from), (len2, from2));
                    let bytes: Vec<u8> = buf[..len]
                        .iter()
                        .map(|b| unsafe { b.assume_init() })
                        .collect();
                    received = Some((bytes, from));
                    break;
                }
                Err(NetworkError::WouldBlock) => std::thread::sleep(Duration::from_millis(20)),
                Err(e) => panic!("recv: {e}"),
            }
        }
        let (bytes, from) = received.expect("the allowed datagram arrives");
        assert_eq!(bytes, b"good");
        assert_eq!(from.ip(), good.local_addr().unwrap().ip());

        // Sending is checked per destination.
        assert_eq!(
            err_of!(socket.try_send_to(b"x", sa("127.0.0.2:9"))),
            NetworkError::PermissionDenied
        );
        assert!(socket.try_send_to(b"x", good.local_addr().unwrap()).is_ok());
        let mut reply = [0u8; 4];
        good.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        assert_eq!(good.recv(&mut reply).unwrap(), 1);

        // Broadcast and multicast need unrestricted access.
        assert_eq!(
            err_of!(socket.set_broadcast(true)),
            NetworkError::PermissionDenied
        );
        assert_eq!(
            err_of!(socket.join_multicast_v4(Ipv4Addr::new(224, 0, 0, 1), Ipv4Addr::UNSPECIFIED)),
            NetworkError::PermissionDenied
        );
        assert_eq!(
            err_of!(socket.join_multicast_v6(Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 1), 0)),
            NetworkError::PermissionDenied
        );
        assert!(socket.set_broadcast(false).is_ok());
    }
}
