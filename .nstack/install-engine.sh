#!/usr/bin/env bash
# Installs the NapkinStack engine this project records, in CI and on a workstation
# (.nstack/handbook/INSTALL.md).
#
# A published version (_commit: vX.Y.Z) comes from PyPI, the only published channel.
# Anything else comes from the framework's repository at that commit, and is
# announced: nstack then says, on every run that judges, that its rules are unpublished.
# A ref that cannot be fetched fails here; no published version is ever used in its place.
set -euo pipefail

# Two lines of the answers file, read with sed alone: a runner may hold any YAML tool, or none.
answer() { sed -n "s/^$1: *['\"]\{0,1\}\([^'\"]*\)['\"]\{0,1\} *$/\1/p" .copier-answers.yml; }
VERSION=$(answer _commit)
ERRORS=$(mktemp)
trap 'rm -f "$ERRORS"' EXIT

# uv tool install, said on stderr as uv said it. D249 (#453): uv removes the former engine before it
# links the new one's executables, and refuses one the bin folder already holds. A link that points
# nowhere — the engine's own, just removed — is deleted, which loses nothing, and the same command
# runs again, uv still guarding every other executable; never --force. A link to anything that
# exists belongs to another installation: nothing is touched, and the command to run is named.
engine_install() {
  local round taken bin link
  for round in 1 2 3 4; do
    if uv tool install "$@" 2>"$ERRORS"; then cat "$ERRORS" >&2; return 0; fi
    cat "$ERRORS" >&2
    taken=$(sed -n 's/^error: Executable already exists: \([^ ]*\).*/\1/p' "$ERRORS" | head -n 1)
    [ -n "$taken" ] || return 1
    bin=$(uv tool dir --bin) || return 1
    link="$bin/$taken"
    if [ -L "$link" ] && [ ! -e "$link" ]; then
      echo "[nstack] $taken pointed at an engine uv removed ($link): that link deleted, installed again."
      rm -f "$link"
      continue
    fi
    echo "FAIL [nstack] $taken already belongs to another installation ($link): it is not overwritten,"
    echo "      and the engine uv removed is not installed until you choose."
    echo "      Action: if the engine's own $taken is the one wanted, run this once:"
    echo "      uv tool install $(printf '%q ' "$@")--force"
    exit 1
  done
  return 1
}
if [[ "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  # --refresh-package: a runner that outlives its jobs keeps uv's cache, and a page of the
  # registry read before a release would say this version does not exist.
  # Minutes after a release, PyPI's index may not list the version its project page shows: a
  # version not listed is asked again, a few times, said; any other failure fails at once.
  for ATTEMPT in 1 2 3 4 5 6; do
    if engine_install "napkinstack==${VERSION#v}" --refresh-package napkinstack --with-executables-from pre-commit; then
      exit 0
    fi
    grep -q "no version of napkinstack" "$ERRORS" && [ "$ATTEMPT" -lt 6 ] || break
    echo "[nstack] napkinstack ${VERSION#v} not listed yet by PyPI's index (attempt $ATTEMPT of 6): asked again in 30 s."
    sleep 30
  done
  if grep -q "no version of napkinstack" "$ERRORS"; then
    # D201: an update made by an engine run from its sources is ahead of the release
    echo "FAIL [nstack] napkinstack ${VERSION#v} could not be installed from PyPI: not published yet — its release"
    echo "      may await approval — or the registry did not answer."
    echo "      Action: run this check again once https://pypi.org/project/napkinstack/${VERSION#v}/ answers."
  else
    echo "FAIL [nstack] napkinstack ${VERSION#v} could not be installed: uv said why, above."
    echo "      Action: fix what uv said, then run this script again."
  fi
  exit 1
fi

SOURCE=$(answer _src_path)
case "$SOURCE" in  # Copier's remote forms, as nstack doctor L7 reads them
  https://*|ssh://*) ;;
  gh:*) SOURCE="https://github.com/${SOURCE#gh:}" ;;
  gl:*) SOURCE="https://gitlab.com/${SOURCE#gl:}" ;;
  git@*:*) HOST=${SOURCE#git@}; SOURCE="ssh://git@${HOST%%:*}/${HOST#*:}" ;;
  *) echo "FAIL [nstack] framework source '$SOURCE' is a path on one machine: CI cannot fetch it."
     echo "      Action: nstack update from the framework's repository."
     exit 1 ;;
esac
REF=${VERSION##*-g}  # Copier records an unpublished commit as git describe: vX.Y.Z-N-g<sha>
echo "::warning::UNPUBLISHED NapkinStack: ${SOURCE}@${REF} judges this run"
engine_install "napkinstack @ git+${SOURCE}@${REF}" --with-executables-from pre-commit || {
  echo "FAIL [nstack] framework ref '$REF' could not be installed from $SOURCE — uv said why, above: no"
  echo "      published version is used in its place."
  echo "      Action: pin an existing commit with nstack update --ref, or a published vX.Y.Z."
  exit 1
}
