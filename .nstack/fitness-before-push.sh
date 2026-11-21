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
# In an agent's session (nstack agent run), nothing is pushed under another identity than the
# agent's own (#456): a commit git made up from the machine would publish its user and host name.
# Read: every commit of the local branches and tags, and of the ref pushed, that the remote does not
# hold yet — pre-commit names only the first ref of a push of several. What a fetch brought — the
# remote-tracking branches, the attested records fetched beside them — is the remote's already.
if [ -n "${NSTACK_AGENT:-}" ]; then
  MINE=$(git config --global user.email)
  NAME=$(git config --global user.name)
  if [ -z "$MINE" ] || [ -z "$NAME" ]; then
    echo "FAIL [nstack] agent ${NSTACK_AGENT} declares no identity in its own git configuration: nothing is pushed."
    echo "      Action: nstack agent run ${NSTACK_AGENT} -- <command> writes it; then amend the commits' author."
    exit 1
  fi
  FROM=""
  case "${PRE_COMMIT_FROM_REF:-}" in ""|*[!0]*) FROM=${PRE_COMMIT_FROM_REF:-} ;; esac
  COUNT=$(git log --format='%ae%x09%ce%x09%an%x09%cn' --branches --tags ${PRE_COMMIT_TO_REF:+"$PRE_COMMIT_TO_REF"} \
      --not --remotes --glob='refs/notes/nstack-remote/*' ${FROM:+"$FROM"} 2>/dev/null |
    awk -F '\t' -v me="$MINE" -v name="$NAME" '$1 != me || $2 != me || $3 != name || $4 != name' | wc -l | tr -d ' ')
  if [ "${COUNT:-0}" -gt 0 ]; then
    echo "FAIL [nstack] ${COUNT} commit(s) the remote does not hold yet are not agent ${NSTACK_AGENT}'s identity"
    echo "      ($NAME <$MINE>) as author or committer: pushed, they would publish another — this machine's"
    echo "      user and host, or a person's name."
    echo "      Action: the agent's own commits under a made-up identity: git rebase --exec 'git commit --amend"
    echo "      --no-edit --reset-author' <their base>. Another's work brought here keeps its author: it is"
    echo "      pushed from that person's own session, never re-signed."
    exit 1
  fi
fi
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
