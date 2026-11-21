# Runbook — send

> `send` runs on the user's computer, started by the shell for one send. There is no server of ours: what
> follows is what a colleague or a maintainer does when a send fails.

## Alerts and responses

Each failure is a `failed` line's code (`posting/v1`), said on the meeting's page.

| Code | Meaning | First action |
|---|---|---|
| `address.refused` | Not https, credentials in it, or this computer or a local network | Paste the channel's `https:` webhook address |
| `meeting.unreadable` | The session's folder, or a file in it, could not be read | Check the folder; nothing was sent |
| `meeting.confidential` | The meeting is said confidential | Nothing to do: it is never sent (PDR-0007); the page offers no send |
| `meeting.no_summary` | No minutes, or none it can read | Write the summary first |
| `network.unreachable` | No connection, the host not found, its certificate refused | Check the connection; nothing was sent |
| `destination.silent` | Sent, no answer within 30 s | Look in the channel before sending again: it may have arrived |
| `destination.gone` | 401, 403, 404, 410 | The webhook was deleted or revoked: set the destination up again |
| `destination.busy` | 429 | Wait `retry_after` seconds |
| `destination.refused` | Any other 3xx or 4xx | Check that the kind matches the address (a Slack address set up as Discord's) |
| `destination.failed` | 5xx | The service is down: try later |
| `summary.too_large` | Beyond Discord's or Slack's bounds | Export it as a file instead |

The log (`KITOKEY_TRACE`) holds the kind, the outcome and the status, never the address: a report never
asks the user for it.

## Rollback

A release of the package without `kitokey-send`, or with the one before: the shell offers no send when
the program is absent (the shell's pull request holds it). Nothing is stored by `send` to roll back.

## Post-deployment verification

After a release: one send to a test channel of each kind (a webhook.site-like receiver, a Discord server
and a Slack workspace of the team's), and one refusal (`https://127.0.0.1/`), on each system.

## Dependencies and degradation

- The network: absent, `network.unreachable`, nothing sent, nothing retried.
- The destination: its answer is read for its status alone; down, `destination.failed`.
- The system's resolver: a name it cannot resolve is `network.unreachable`.
