//! posting/v1's *Which addresses it refuses*: every range of the page refused, a public address kept,
//! an IPv6 carrying an IPv4 judged as that IPv4, the address read as the connecting parser reads it.

use std::net::IpAddr;

use kitokey_send::{Refusal, judged, local};

fn ip(text: &str) -> IpAddr {
    text.parse().unwrap()
}

#[test]
fn every_range_of_the_page_is_local_at_its_edges_and_a_public_address_is_not() {
    let local_ones = [
        "127.0.0.1",
        "127.255.255.255",
        "::1",
        "0.0.0.0",
        "0.255.255.255",
        "::",
        "10.0.0.0",
        "10.255.255.255",
        "172.16.0.0",
        "172.31.255.255",
        "192.168.0.0",
        "192.168.255.255",
        "100.64.0.0",
        "100.127.255.255",
        "169.254.0.0",
        "169.254.255.255",
        "fe80::",
        "febf::1",
        "fc00::",
        "fdff::1",
        "224.0.0.0",
        "239.255.255.255",
        "ff00::",
        "ff02::1",
        "255.255.255.255",
        "64:ff9b:1::",
        "64:ff9b:1:ffff::1",
        // Carrying an IPv4: judged as it.
        "::ffff:127.0.0.1",
        "::ffff:10.1.2.3",
        "64:ff9b::127.0.0.1",
        "64:ff9b::a9fe:a9fe",
    ];
    let public = [
        "1.1.1.1",
        "9.255.255.255",
        "11.0.0.0",
        "172.15.255.255",
        "172.32.0.0",
        "192.167.255.255",
        "192.169.0.0",
        "100.63.255.255",
        "100.128.0.0",
        "169.253.255.255",
        "223.255.255.255",
        "2606:4700:4700::1111",
        "fec0::1",
        "fe7f::1",
        "fbff::1",
        "64:ff9b:2::1",
        "::ffff:1.1.1.1",
        "64:ff9b::1.1.1.1",
    ];
    let wrong: Vec<_> = local_ones.iter().filter(|text| !local(ip(text))).collect();
    assert!(wrong.is_empty(), "taken for public: {wrong:?}");
    let wrong: Vec<_> = public.iter().filter(|text| local(ip(text))).collect();
    assert!(wrong.is_empty(), "taken for local: {wrong:?}");
}

#[test]
fn an_address_not_https_holding_credentials_or_a_local_literal_is_refused_before_any_resolution() {
    let cases = [
        ("http://discord.com/api/webhooks/1/a", Refusal::NotHttps),
        ("ftp://discord.com/x", Refusal::NotHttps),
        ("discord.com/api/webhooks/1/a", Refusal::Unreadable),
        ("", Refusal::Unreadable),
        ("https://user:secret@discord.com/x", Refusal::Credentials),
        ("https://user@discord.com/x", Refusal::Credentials),
        ("https://127.0.0.1/x", Refusal::Local),
        ("https://[::1]/x", Refusal::Local),
        ("https://[::ffff:192.168.1.1]/x", Refusal::Local),
        // Read as a browser reads it: hex and decimal IPv4.
        ("https://0x7f.1/x", Refusal::Local),
        ("https://2130706433/x", Refusal::Local),
        ("https://0177.0.0.1/x", Refusal::Local),
    ];
    for (address, refusal) in cases {
        assert_eq!(judged(address).err(), Some(refusal), "{address}");
    }
}

#[test]
fn a_name_is_kept_lowercased_and_as_punycode_its_path_whole() {
    let kept = judged("https://Hooks.Slack.COM/services/T0/B0/xyz").unwrap();
    assert_eq!(kept.as_str(), "https://hooks.slack.com/services/T0/B0/xyz");
    let kept = judged("https://bücher.example/hook").unwrap();
    assert_eq!(kept.host_str(), Some("xn--bcher-kva.example"));
    let kept = judged("https://1.1.1.1/hook?x=1").unwrap();
    assert_eq!(kept.as_str(), "https://1.1.1.1/hook?x=1");
}

#[test]
fn a_name_answering_one_local_address_among_public_ones_is_refused_whole() {
    use kitokey_send::{Policy, admitted};
    let public: std::net::SocketAddr = "1.1.1.1:443".parse().unwrap();
    let here: std::net::SocketAddr = "[::1]:443".parse().unwrap();
    let strict = Policy::strict().reachable;
    assert!(admitted(&[public], strict));
    assert!(!admitted(&[public, here], strict));
    assert!(!admitted(&[here, public], strict));
}

#[test]
fn the_programs_policy_is_https_alone_within_30_seconds_and_no_local_address() {
    use kitokey_send::Policy;
    let strict = Policy::strict();
    assert!(strict.https_only);
    assert_eq!(strict.bound, std::time::Duration::from_secs(30));
    assert!(!(strict.reachable)(ip("10.0.0.1")) && (strict.reachable)(ip("1.1.1.1")));
}
