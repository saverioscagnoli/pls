package file

import (
	"io/fs"
	"iter"
	"os"
	"path/filepath"
	"slices"
)

type Entry struct {
	Path  string
	Depth uint
	fs.DirEntry
}

func Walk(root string, maxDepth uint, all bool) iter.Seq2[Entry, error] {
	return func(yield func(Entry, error) bool) {
		info, err := os.Lstat(root)
		if err != nil {
			yield(Entry{Path: root}, err)
			return
		}

		if target, err := os.Stat(root); err == nil && target.IsDir() {
			walk(root, 1, maxDepth, all, yield)
			return
		}

		yield(Entry{Path: root, Depth: 1, DirEntry: fs.FileInfoToDirEntry(info)}, nil)
	}
}

func walk(dir string, depth, maxDepth uint, all bool, yield func(Entry, error) bool) bool {
	entries, err := os.ReadDir(dir)
	if err != nil {
		return yield(Entry{Path: dir, Depth: depth}, err)
	}

	// Directories first
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
		path := filepath.Join(dir, entry.Name())

		if !all && entry.Name()[0] == '.' {
			continue
		}

		if !yield(Entry{Path: path, Depth: depth, DirEntry: entry}, nil) {
			return false
		}

		if entry.IsDir() && depth < maxDepth {
			if !walk(path, depth+1, maxDepth, all, yield) {
				return false
			}
		}
	}

	return true
}
