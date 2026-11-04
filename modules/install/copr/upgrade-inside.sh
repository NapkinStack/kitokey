#!/usr/bin/env bash
# D78, in copr/upgrade.sh's container, Kitokey installed from Copr: `before`, a user's first use of
# that version, the home kept as it then is; `after`, `dnf upgrade` — the next version installed,
# each program saying it, the home as it was, and Kitokey started again reading the meetings and
# the settings it left. Run as root, given $tools, $driver and $version, the next one's.
set -uo pipefail
source /as-user.sh
case ${1:?before or after} in
  before)
    check "kitokey $(rpm -q --qf '%{VERSION}' kitokey) installed from Copr" "rpm -q kitokey >/dev/null"
    first_use
    snapshot > /tmp/kept-before
    check "Kitokey is not running before the upgrade" "! pgrep -x kitokey >/dev/null" ;;
  after)
    previous=$(rpm -q --qf '%{VERSION}' kitokey)
    check "upgraded with dnf, kitokey $previous to $version" "dnf upgrade -y -q --refresh kitokey >/tmp/upgrade.log 2>&1 && test \"\$(rpm -q --qf '%{VERSION}' kitokey)\" = \"\$version\" || { tail -5 /tmp/upgrade.log; false; }"
    for program in kitokey kitokey-capture kitokey-transcription kitokey-summary; do
      check "$program says $version" "test \"\$($program --version)\" = \"$program $version\""
    done
    check "the meetings and the settings stay as they were" "test \"\$(snapshot)\" = \"\$(cat /tmp/kept-before)\""
    reads_again ;;
esac
exit $failed
