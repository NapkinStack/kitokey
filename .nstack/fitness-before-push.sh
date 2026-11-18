#!/bin/sh
# Before a push, the fitness functions CI runs on every pull request: the pre-push hook of
# .pre-commit-config.yaml. Judged by an engine at the project's version or ahead of it, never
# behind: an older engine refuses what the project's version defines, and can pass what it
# refuses. An engine ahead is where an update starts. A project pinned to an unpublished commit
# (vX.Y.Z-N-g<sha>) is ahead of vX.Y.Z: an engine behind that is refused; one at it or ahead
# judges, and every run of nstack says what it is. A version this cannot read — no answers
# file, a pre-release — leaves the judging to the engine here, as before, and to CI.
#
# Plain sh, with sed and awk: the workstation's engine may predate this file.
# What nstack records beside the code — a hold's event (claim, unclaim, deliver) on
# refs/heads/nstack/hold/…, an attested record in refs/notes/nstack/… — moves no file of the
# project: a red default branch never stops it being recorded (#427). Every other hook still
# runs on it, and CI judges every pull request.
case "${PRE_COMMIT_REMOTE_BRANCH:-}" in
  refs/heads/nstack/hold/*|refs/notes/nstack/*) echo "nstack's own record: no file of the project to judge."; exit 0 ;;
esac
command -v nstack >/dev/null 2>&1 || { echo "nstack is not installed: CI judges"; exit 0; }

RECORDED=$([ -f .copier-answers.yml ] && tr -d '\r' < .copier-answers.yml |
  sed -n "s/^_commit: *['\"]\{0,1\}\(v[0-9]*\.[0-9]*\.[0-9]*\(-[0-9]*-g[0-9a-f]*\)\{0,1\}\)['\"]\{0,1\} *$/\1/p")
WANTED=${RECORDED%%-*}
WANTED=${WANTED#v}
HERE=$(nstack --version | sed -n 's/^nstack \([0-9]*\.[0-9]*\.[0-9]*\).*/\1/p')
if [ -n "$WANTED" ] && [ -n "$HERE" ] && awk -v here="$HERE" -v wanted="$WANTED" 'BEGIN {
    split(here, h, "."); split(wanted, w, ".")
    for (i = 1; i <= 3; i++) { if (h[i] + 0 < w[i] + 0) exit 0; if (h[i] + 0 > w[i] + 0) exit 1 }
    exit 1 }'; then
  echo "FAIL [nstack] This workstation runs nstack $HERE; the project is on $RECORDED. Its rules are not"
  echo "      the project's: it would refuse what $RECORDED defines, and could pass what it refuses."
  echo "      Action: bash .nstack/install-engine.sh   # the version the project records, then push again"
  exit 1
fi

for check in manifests boundaries plan hygiene; do
  nstack "$check" --root . || exit 1
done
