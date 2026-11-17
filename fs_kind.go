package main

import (
	"io/fs"
	"os"
)

type DirEntryKind string

const (
	DirEntryKindFile             DirEntryKind = "file"
	DirEntryKindDirectory        DirEntryKind = "directory"
	DirEntryKindSymlinkFile      DirEntryKind = "symlink-file"
	DirEntryKindSymlinkDirectory DirEntryKind = "symlink-directory"
	DirEntryKindBrokenSymlink    DirEntryKind = "broken-symlink"
)

func (e *DirEntryExt) Kind() DirEntryKind {
	t := e.Type()

	if t&fs.ModeSymlink == 0 {
		if t.IsDir() {
			return DirEntryKindDirectory
		}

		return DirEntryKindFile
	}

	target, err := os.Stat(e.Path)
	if err != nil {
		return DirEntryKindBrokenSymlink
	}

	if target.IsDir() {
		return DirEntryKindSymlinkDirectory
	}

	return DirEntryKindSymlinkFile
}
