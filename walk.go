package main

import (
	"io/fs"
	"iter"
	"os"
	"path/filepath"
	"slices"
)

type DirEntryExt struct {
	Path  string
	Depth uint
	fs.DirEntry
}

func walkInner(root string, depth uint, maxDepth uint, all bool, yield func(DirEntryExt, error) bool) bool {
	entries, err := os.ReadDir(root)
	if err != nil {
		return yield(DirEntryExt{Path: root, Depth: depth}, err)
	}

	slices.SortStableFunc(entries, func(a, b fs.DirEntry) int {
		switch {
		case a.IsDir() && !b.IsDir():
			return -1
		case !a.IsDir() && b.IsDir():
			return 1
		}

		return 0
	})

	for _, entry := range entries {
		path := filepath.Join(root, entry.Name())

		if !all && entry.Name()[0] == '.' {
			continue
		}

		if !yield(DirEntryExt{Path: path, Depth: depth, DirEntry: entry}, nil) {
			return false
		}

		if entry.IsDir() && depth < maxDepth {
			if !walkInner(path, depth+1, maxDepth, all, yield) {
				return false
			}
		}
	}

	return true
}

func Walk(root string, maxDepth uint, all bool) iter.Seq2[DirEntryExt, error] {
	return func(yield func(DirEntryExt, error) bool) {
		info, err := os.Lstat(root)
		if err != nil {
			yield(DirEntryExt{Path: root}, err)
			return
		}

		if target, err := os.Stat(root); err == nil && target.IsDir() {
			walkInner(root, 1, maxDepth, all, yield)
			return
		}

		yield(DirEntryExt{Path: root, Depth: 1, DirEntry: fs.FileInfoToDirEntry(info)}, nil)
	}
}
