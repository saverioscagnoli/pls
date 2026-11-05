#!/usr/bin/env bash
# Removes fixtures, restoring permissions first so rm can descend.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="${1:-$here/fixtures}"
[ -e "$root" ] || exit 0
chmod -R u+rwx "$root" 2>/dev/null || true
rm -rf "$root"
