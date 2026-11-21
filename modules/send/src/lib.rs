//! `kitokey-send`'s core, as `posting/v1` says: which addresses are refused, and the one POST. The
//! program uses [`Policy::strict`] alone; a test injects its own policy through this library to reach a
//! server on this computer, never through the program.

use std::fmt;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;

use ureq::Agent;
use ureq::unversioned::resolver::{DefaultResolver, ResolvedSocketAddrs, Resolver};
use ureq::unversioned::transport::{DefaultConnector, NextTimeout};
use url::{Host, Url};

// ---------------------------------------------------------------------------------------------------
// Which addresses it refuses

/// Why an address is refused (`address.refused`), said to the person in the program's sentence — which
/// never holds the address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Unreadable,
    NotHttps,
    Credentials,
    Local,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Refusal::Unreadable => "the address is not a web address",
            Refusal::NotHttps => "the address is not https",
            Refusal::Credentials => "the address holds a user or a password",
            Refusal::Local => "the address is this computer's or a local network's",
        })
    }
}

/// The address as the connection reads it — a name lowercased and as punycode, an IPv4 written in hex or
/// decimal as the address it stands for (WHATWG's parser) —, or why it is refused. A name is judged again
/// on every address it resolves to, when it is sent ([`post`]).
pub fn judged(address: &str) -> Result<Url, Refusal> {
    let url = Url::parse(address).map_err(|_| Refusal::Unreadable)?;
    if url.scheme() != "https" {
        return Err(Refusal::NotHttps);
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(Refusal::Credentials);
    }
    let ip = match url.host() {
        None => return Err(Refusal::Unreadable),
        Some(Host::Ipv4(ip)) => Some(IpAddr::V4(ip)),
        Some(Host::Ipv6(ip)) => Some(IpAddr::V6(ip)),
        Some(Host::Domain(_)) => None,
    };
    if ip.is_some_and(local) {
        return Err(Refusal::Local);
    }
    Ok(url)
}

/// Whether an address is this computer's or a local network's: the page's ranges, an IPv6 carrying an
/// IPv4 judged as that IPv4.
pub fn local(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => local_v4(ip),
        IpAddr::V6(ip) => match carried(ip) {
            Some(v4) => local_v4(v4),
            None => {
                let first = ip.segments()[0];
                ip.is_loopback()
                    || ip.is_unspecified()
                    || first & 0xffc0 == 0xfe80
                    || first & 0xfe00 == 0xfc00
                    || ip.is_multicast()
                    || ip.segments()[..3] == [0x64, 0xff9b, 1]
            }
        },
    }
}

fn local_v4(ip: Ipv4Addr) -> bool {
    let [a, b, ..] = ip.octets();
    ip.is_loopback()
        || a == 0
        || ip.is_private()
        || (a == 100 && b & 0xc0 == 64)
        || ip.is_link_local()
        || ip.is_multicast()
        || ip.is_broadcast()
}

/// The IPv4 address an IPv6 one carries: mapped (`::ffff:0:0/96`), or NAT64's well-known prefix
/// (`64:ff9b::/96`).
fn carried(ip: Ipv6Addr) -> Option<Ipv4Addr> {
    let segments = ip.segments();
    if segments[..6] == [0, 0, 0, 0, 0, 0xffff] || segments[..6] == [0x64, 0xff9b, 0, 0, 0, 0] {
        let [.., a, b, c, d] = ip.octets();
        return Some(Ipv4Addr::new(a, b, c, d));
    }
    None
}

// ---------------------------------------------------------------------------------------------------
// The one POST

/// What a send may reach. The program's is [`Policy::strict`]; a test's reaches a server on this computer.
#[derive(Clone, Copy, Debug)]
pub struct Policy {
    /// Whether a resolved address may be connected to.
    pub reachable: fn(IpAddr) -> bool,
    /// `https:` alone.
    pub https_only: bool,
    /// The whole send: connection, request and answer.
    pub bound: Duration,
}

impl Policy {
    /// `posting/v1`'s: no local address, `https:` alone, 30 s.
    pub fn strict() -> Self {
        Policy {
            reachable: |ip| !local(ip),
            https_only: true,
            bound: Duration::from_secs(30),
        }
    }
}

/// A request's body and its type.
#[derive(Clone, Debug, PartialEq)]
pub struct Body {
    pub content_type: String,
    pub bytes: Vec<u8>,
}

/// How a send ended, each a code of `posting/v1` but [`Outcome::Sent`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Sent,
    /// A resolved address was local: nothing sent.
    Refused,
    /// No connection: nothing sent.
    Unreachable,
    /// Sent, and no answer within the bound: maybe arrived.
    Silent,
    /// 401, 403, 404, 410.
    Gone,
    /// 429, and the seconds the destination said to wait, if it said them.
    Busy(Option<u64>),
    /// Any other 3xx or 4xx: a redirect is never followed.
    Declined(u16),
    /// 5xx, or a status no class names.
    Failed(u16),
}

impl Outcome {
    /// The code of `posting/v1`'s `failed` line, none for [`Outcome::Sent`].
    pub fn code(self) -> Option<&'static str> {
        Some(match self {
            Outcome::Sent => return None,
            Outcome::Refused => "address.refused",
            Outcome::Unreachable => "network.unreachable",
            Outcome::Silent => "destination.silent",
            Outcome::Gone => "destination.gone",
            Outcome::Busy(_) => "destination.busy",
            Outcome::Declined(_) => "destination.refused",
            Outcome::Failed(_) => "destination.failed",
        })
    }
}

/// A name resolved once, every address judged; the connection made to those, never to a second
/// resolution. One local address refuses them all.
#[derive(Debug)]
struct Judging {
    resolver: DefaultResolver,
    reachable: fn(IpAddr) -> bool,
}

/// The resolver's refusal, told apart from the network's by its type.
#[derive(Debug)]
struct LocalAddress;

impl fmt::Display for LocalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a resolved address is local")
    }
}

impl std::error::Error for LocalAddress {}

/// The resolver's failure to resolve the name: nothing was sent.
#[derive(Debug)]
struct Unresolved;

impl fmt::Display for Unresolved {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("the host could not be resolved")
    }
}

impl std::error::Error for Unresolved {}

impl Resolver for Judging {
    fn resolve(
        &self,
        uri: &ureq::http::Uri,
        config: &ureq::config::Config,
        timeout: NextTimeout,
    ) -> Result<ResolvedSocketAddrs, ureq::Error> {
        // A name not resolved: nothing left, whatever error the system gave for it.
        let addresses = self
            .resolver
            .resolve(uri, config, timeout)
            .map_err(|_| ureq::Error::Other(Box::new(Unresolved)))?;
        if !admitted(&addresses, self.reachable) {
            return Err(ureq::Error::Other(Box::new(LocalAddress)));
        }
        Ok(addresses)
    }
}

/// Whether a name's resolved addresses may be connected to: every one of them reachable, so that a name
/// answering a public address and a local one is refused whole.
pub fn admitted(addresses: &[SocketAddr], reachable: fn(IpAddr) -> bool) -> bool {
    addresses.iter().all(|address| reachable(address.ip()))
}

/// Sends `body` once to `address`, judged first ([`judged`]): no proxy, no redirect, within the bound.
/// Only the answer's status, and its `Retry-After`, are read; never its body.
pub fn post(address: &Url, body: &Body, policy: Policy) -> Outcome {
    let config = Agent::config_builder()
        .proxy(None)
        .max_redirects(0)
        .http_status_as_error(false)
        .https_only(policy.https_only)
        .timeout_global(Some(policy.bound))
        .build();
    let agent = Agent::with_parts(
        config,
        DefaultConnector::default(),
        Judging {
            resolver: DefaultResolver::default(),
            reachable: policy.reachable,
        },
    );
    let answer = agent
        .post(address.as_str())
        .header("Content-Type", &body.content_type)
        .send(&body.bytes[..]);
    match answer {
        Ok(answer) => {
            let status = answer.status().as_u16();
            match status {
                200..=299 => Outcome::Sent,
                401 | 403 | 404 | 410 => Outcome::Gone,
                429 => Outcome::Busy(
                    answer
                        .headers()
                        .get("retry-after")
                        .and_then(|value| value.to_str().ok())
                        .and_then(|value| value.trim().parse().ok()),
                ),
                300..=499 => Outcome::Declined(status),
                _ => Outcome::Failed(status),
            }
        }
        Err(ureq::Error::Other(error)) if error.is::<LocalAddress>() => Outcome::Refused,
        Err(ureq::Error::Other(error)) if error.is::<Unresolved>() => Outcome::Unreachable,
        Err(error) => failed(&error),
    }
}

/// Whether a failure came before anything was sent, or after: only a failure to resolve or to connect
/// says that nothing left.
fn failed(error: &ureq::Error) -> Outcome {
    use ureq::Error::*;
    match error {
        HostNotFound | ConnectionFailed | Tls(_) | Rustls(_) => Outcome::Unreachable,
        Timeout(ureq::Timeout::Resolve | ureq::Timeout::Connect) => Outcome::Unreachable,
        Io(error)
            if matches!(
                error.kind(),
                io::ErrorKind::ConnectionRefused
                    | io::ErrorKind::NotFound
                    | io::ErrorKind::AddrNotAvailable
                    | io::ErrorKind::NetworkUnreachable
                    | io::ErrorKind::HostUnreachable
            ) =>
        {
            Outcome::Unreachable
        }
        _ => Outcome::Silent,
    }
}
