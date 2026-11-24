"""posting/v1's security promises (D110, PDR-0007), held on its page: the address ranges it refuses,
no redirect and no proxy followed, the address never in a sentence or a log, Discord's mentions off.
A promise taken out of the page, or a range removed, fails here."""

import ipaddress
import re
from pathlib import Path

PAGE = (Path(__file__).parent.parent / "posting/v1/README.md").read_text()

# Refused outright: this computer, a local network, and what no channel's webhook is.
REFUSED = {
    "127.0.0.0/8",
    "::1/128",
    "0.0.0.0/8",
    "::/128",
    "10.0.0.0/8",
    "172.16.0.0/12",
    "192.168.0.0/16",
    "100.64.0.0/10",
    "169.254.0.0/16",
    "fe80::/10",
    "fc00::/7",
    "224.0.0.0/4",
    "ff00::/8",
    "255.255.255.255/32",
    "64:ff9b:1::/48",
}
# Judged as the IPv4 address they carry.
CARRYING = {"::ffff:0:0/96", "64:ff9b::/96"}


def section(heading):
    return PAGE.split(f"## {heading}\n", 1)[1].split("\n## ", 1)[0]


def ranges(text):
    return {token for token in re.findall(r"`([0-9a-f:.]+/\d+)`", text) if ipaddress.ip_network(token)}


def test_the_page_refuses_every_local_range_and_judges_those_carrying_an_ipv4_as_it():
    refusing = section("Which addresses it refuses")
    listed, carried = refusing.split("An IPv6 address that carries an IPv4 one", 1)
    assert ranges(listed) == REFUSED
    assert ranges(carried.split("\n\n", 1)[0]) == CARRYING
    assert "**Every** address the host resolves to is judged" in refusing
    assert "never to a second resolution" in refusing
    # Each is a network as written: no host bits set.
    assert all(ipaddress.ip_network(token, strict=True) for token in REFUSED | CARRYING)


def test_the_page_follows_no_redirect_and_no_proxy():
    refusing = section("Which addresses it refuses")
    assert "A redirect is never followed" in refusing
    assert "No proxy is followed" in refusing
    assert all(name in refusing for name in ("`HTTPS_PROXY`", "`ALL_PROXY`"))


def test_the_page_never_puts_the_address_in_a_sentence_a_log_or_the_arguments():
    flat = " ".join(PAGE.split())
    assert "The sentence never holds the address, nor any part of it" in flat
    assert "**never the address, nor any part of it**" in flat
    assert "The arguments hold no word of the meeting and no address" in flat


def test_the_page_turns_discords_mentions_off():
    assert '`allowed_mentions` `{"parse": []}`' in section("What it sends")


def test_notion_pins_its_version_and_reaches_its_api_alone():
    notion = section("Notion")
    assert "`Notion-Version: 2026-03-11`" in notion
    assert "goes to `https://api.notion.com`" in notion
    assert "reaches Notion's API, never the address" in notion
    assert "**one**\n  `POST /v1/pages`" in notion or "**one** `POST /v1/pages`" in " ".join(notion.split())


def test_notion_holds_its_documented_limits_and_creates_one_page_or_none():
    flat = " ".join(section("Notion").split())
    for limit in ("at most 2000 characters", "at most 100 rich texts", "at most 100 blocks in `children`", "500 KB"):
        assert limit in flat, limit
    assert "A summary that does not fit **one** request is `summary.too_large`, and nothing is created" in flat
    assert "each action a `to_do`, left unticked" in flat


def test_notion_has_a_code_for_each_refusal_a_person_acts_on_differently():
    codes = section("What it carries")
    for code in ("key.refused", "database.unshared", "key.restricted", "database.several"):
        assert f"| `{code}` |" in codes, code
    assert "`database.unshared` names both" in " ".join(section("Notion").split())


def test_the_key_is_a_secret_as_the_address_is():
    flat = " ".join(PAGE.split())
    assert "the integration's key, as the person pasted it — a secret, as an address is" in flat
    assert "never the key" in flat


def test_notion_reads_the_database_as_a_link_holds_it_and_says_what_an_unanswered_read_is():
    flat = " ".join(section("Notion").split())
    assert "the link's last 32 hexadecimal characters before any `?`, hyphens left out" in flat
    assert "holds the database's title before them" in flat
    assert "| No answer to a read (`GET`) within the bound | `destination.failed` | Nothing |" in section("Notion")
    assert "a database holding several is `database.several`, nothing created" in flat
