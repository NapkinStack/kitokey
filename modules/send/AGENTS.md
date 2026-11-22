# send — local instructions

> Only what is **specific to this module**. Indicative budget: 100 lines.

## What this module does not do

- It writes nothing: no file, no record of a send. The record — to which destination, when — is the
  shell's (PDR-0007, decision 5). The session's folder is only read.
- It keeps no address: the shell keeps it in the system's keyring and gives it on the input, for one send.
- It writes no file format: a file is `export`'s (`exporting/v1`).

## Business invariants

- **The address is a secret**: never in an argument, a sentence, a line or a log. `Refusal`'s sentences
  hold none of it; the program's tests check every run's output for it (D110's next pull request).
- **A confidential meeting is never sent**, judged before the address (the program, next pull request).
- **Every resolved address is judged, the connection made to those**: `Judging`, the resolver ureq is
  given, refuses a name whose addresses hold one local address; there is no second resolution
  (`tests/address.rs`, `tests/server.rs`).
- **No proxy, no redirect, one POST**: `.proxy(None)`, `.max_redirects(0)`, never a retry
  (`tests/server.rs`, each killed by its mutant).
- **The program uses `Policy::strict()` alone.** A test reaches this computer by a policy of its own,
  through the library; no flag of the program, no variable, loosens it.
- Only the answer's status and `Retry-After` are read; never its body.

## Known traps

- The machine's `NO_PROXY` exempts this computer from a proxy: a test of the proxy clears it, or it
  proves nothing (`no_proxy_of_the_environment_is_followed`).
- A name the system cannot resolve fails as a generic I/O error: `Judging` marks it `Unresolved`, else it
  would read « maybe arrived » (`a_name_not_resolved_is_unreachable`).
- ureq's resolver API is « unversioned »: an upgrade of ureq is read against `Judging` and its tests.
