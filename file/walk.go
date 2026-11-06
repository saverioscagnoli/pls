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
		walk(root, 1, maxDepth, all, yield)
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
