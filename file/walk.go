package file

import (
	"io/fs"
	"iter"
	"os"
	"path/filepath"
	"strings"
)

type Entry struct {
	Path  string
	Depth uint
	fs.DirEntry
}

func Walk(root string, maxDepth uint, all bool) iter.Seq2[Entry, error] {
	return func(yield func(Entry, error) bool) {
		target, err := os.Stat(root)

		if err != nil || !target.IsDir() {
			info, lerr := os.Lstat(root)

			if lerr != nil {
				yield(Entry{Path: root}, lerr)
				return
			}

			yield(Entry{Path: root, DirEntry: fs.FileInfoToDirEntry(info)}, nil)
			return
		}

		sep := string(filepath.Separator)

		if info, err := os.Lstat(root); err == nil && info.Mode()&fs.ModeSymlink != 0 && !strings.HasSuffix(root, sep) {
			root += sep
		}

		filepath.WalkDir(root, func(path string, d fs.DirEntry, err error) error {
			if err != nil {
				if !yield(Entry{Path: path}, err) {
					return filepath.SkipAll
				}

				return nil
			}

			if path == root {
				if maxDepth == 0 {
					return filepath.SkipDir
				}
				return nil
			}

			if !all && strings.HasPrefix(d.Name(), ".") {
				if d.IsDir() {
					return filepath.SkipDir
				}
				return nil
			}

			rel, _ := filepath.Rel(root, path)
			depth := uint(strings.Count(rel, sep)) + 1

			if !yield(Entry{Path: path, Depth: depth, DirEntry: d}, nil) {
				return filepath.SkipAll
			}

			if d.IsDir() && depth >= maxDepth {
				return filepath.SkipDir
			}

			return nil
		})
	}
}

func Walk(root string, maxDepth uint, all bool) iter.Seq2[Entry, error] {
	return func(yield func(Entry, error) bool) {
		entries, err := os.ReadDir(root)

		if err != nil {
			if !yield(Entry{}, err) {
				return
			}
		}

		for _, entry := range entries {
		}
	}
}
