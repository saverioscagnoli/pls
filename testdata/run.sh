#!/usr/bin/env bash
# Builds pls and runs it against every fixture, printing output + exit code.
# Usage: test/run.sh [extra pls args...]   e.g. test/run.sh -a
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
fx="$here/fixtures"
bin="$here/.pls-test"

[ -d "$fx" ] || "$here/setup.sh"
(cd "$here/.." && go build -o "$bin" .) || exit 1

run() {
  printf '\n\033[1;36m━━ pls %s\033[0m\n' "$*"
  "$bin" "$@"
  local rc=$?
  [ $rc -ne 0 ] && printf '\033[1;31m→ exit %d\033[0m\n' $rc
}

cd "$fx"
for d in basic empty hidden symlinks permissions special names sizes times hardlinks; do
  run "$d" "$@"
done
run hidden -a
run symlinks -d 3 "$@"
run deep -d 100 "$@"
run nested-denied -d 5 "$@"
run denied-root "$@"
run nostat-root "$@"
run basic/file.txt "$@"
run symlink-root "$@"
run broken-root "$@"
run does-not-exist "$@"
run many "$@" | tail -3
