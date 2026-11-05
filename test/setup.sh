#!/usr/bin/env bash
# Generates fixtures for testing `pls` list. Re-runnable: wipes and rebuilds.
# Usage: test/setup.sh [dir]   (default: test/fixtures)
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
root="${1:-$here/fixtures}"

"$here/clean.sh" "$root"
mkdir -p "$root"
cd "$root"

# --- basic: plain files, dirs, extensions ------------------------------------
mkdir basic && (
  cd basic
  echo hello > file.txt
  echo 'fn main() {}' > main.rs
  echo 'package main' > main.go
  echo '{}' > data.json
  touch Makefile README.md noext
  mkdir dir
  echo '#!/bin/sh' > script.sh && chmod +x script.sh
)

# --- empty dir ----------------------------------------------------------------
mkdir empty

# --- hidden: dotfiles only (try with and without -a) --------------------------
mkdir hidden && (
  cd hidden
  touch .env .gitignore
  mkdir .config && touch .config/inner
  touch visible
)

# --- symlinks: valid, broken, loops, chains, relative/absolute ----------------
mkdir symlinks && (
  cd symlinks
  echo target > real-file
  mkdir real-dir && touch real-dir/inside
  ln -s real-file link-to-file
  ln -s real-dir link-to-dir
  ln -s "$PWD/real-file" absolute-link
  ln -s does-not-exist broken-link
  ln -s /nonexistent/path/x broken-absolute
  ln -s self self-loop
  ln -s loop-b loop-a
  ln -s loop-a loop-b
  ln -s link-to-file chain-1          # chain-1 -> link-to-file -> real-file
  ln -s chain-1 chain-2
  ln -s broken-link chain-to-broken   # chain ending in broken link
  ln -s .. link-to-parent             # can cause infinite recursion with depth
  ln -s . link-to-self-dir
  mkdir gone && ln -s gone dangling-after-delete && rmdir gone
)

# --- permissions: unreadable/untraversable files and dirs ---------------------
mkdir permissions && (
  cd permissions
  echo secret > no-read-file && chmod 000 no-read-file
  echo x > read-only && chmod 444 read-only
  echo x > write-only && chmod 222 write-only
  echo x > exec-only && chmod 111 exec-only

  mkdir no-perms-dir && touch no-perms-dir/hidden && chmod 000 no-perms-dir      # can't list or enter
  mkdir no-exec-dir && touch no-exec-dir/a no-exec-dir/b && chmod 644 no-exec-dir # can list names, can't stat
  mkdir no-read-dir && touch no-read-dir/a && chmod 311 no-read-dir              # can enter, can't list

  echo x > setuid && chmod 4755 setuid
  echo x > setgid && chmod 2755 setgid
  mkdir sticky-dir && chmod 1777 sticky-dir
  echo x > all-perms && chmod 777 all-perms
)

# Unreadable dir as the root itself (pls test/fixtures/denied-root)
mkdir denied-root && touch denied-root/x && chmod 000 denied-root

# Readable dir whose children can't be stat'd (pls test/fixtures/nostat-root)
mkdir nostat-root && touch nostat-root/a nostat-root/b nostat-root/c && chmod 644 nostat-root

# --- special files: fifo, socket ---------------------------------------------
mkdir special && (
  cd special
  mkfifo fifo
  python3 -c 'import socket,sys; s=socket.socket(socket.AF_UNIX); s.bind(sys.argv[1])' sock
  ln -s /dev/null link-to-devnull
  ln -s /dev/tty link-to-chardev
)

# --- weird names --------------------------------------------------------------
mkdir names && (
  cd names
  touch -- 'with spaces' 'tab	char' $'new\nline' '-dash-start' '--double-dash'
  touch 'émoji-🦀.rs' '日本語.txt' 'ÄÖÜ' $'\e[31mansi-escape' 'quote"s' "single'quote"
  touch '*glob*' '$dollar' 'back\slash' '{brace}' '%percent%'
  touch "$(printf 'a%.0s' {1..255})"     # max length filename
  touch -- .hidden.with.dots ..double-dot-start file. trailing-space\ 
  touch UPPER lower MiXeD 10.txt 2.txt 1.txt
)

# --- sizes & times ------------------------------------------------------------
mkdir sizes && (
  cd sizes
  touch zero
  printf x > one-byte
  head -c 1024 /dev/zero > 1KiB
  head -c 1048576 /dev/zero > 1MiB
  truncate -s 10G sparse-10G
  truncate -s 1T sparse-1T
)

mkdir times && (
  cd times
  touch -d '1970-01-01 00:00:00 UTC' epoch
  touch -d '1901-12-14 00:00:00 UTC' pre-epoch 2>/dev/null || true
  touch -d '2038-01-20 00:00:00 UTC' y2038
  touch -d '2999-12-31 23:59:59 UTC' far-future
  touch -d '1 minute ago' recent
  touch now
)

# --- hardlinks (nlink column) -------------------------------------------------
mkdir hardlinks && (
  cd hardlinks
  echo x > original
  ln original link-1
  ln original link-2
  mkdir subdir-a subdir-b      # parent nlink grows with subdirs
)

# --- depth: deep nesting ------------------------------------------------------
mkdir -p deep/1/2/3/4/5/6/7/8/9/10
for d in deep deep/1 deep/1/2/3 deep/1/2/3/4/5/6/7/8/9/10; do touch "$d/file-at-$(basename "$d")"; done

# Permission error mid-walk (only hit with -d >= 2)
mkdir -p nested-denied/ok/denied && touch nested-denied/ok/denied/x nested-denied/ok/fine && chmod 000 nested-denied/ok/denied

# --- many files ---------------------------------------------------------------
mkdir many && (cd many && for i in $(seq 1 2000); do : > "file-$i"; done)

# --- a file passed as the root (pls test/fixtures/basic/file.txt) -------------
# (reuses basic/file.txt; also a symlink as root)
ln -s basic symlink-root
ln -s nope broken-root

echo "fixtures created in $root"
