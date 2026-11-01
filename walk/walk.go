package walk

import (
	"io/fs"
	"iter"
	"path/filepath"
	"strings"
)

type Entry struct {
	Path  string
	Depth uint
	fs.DirEntry
}

func Walk(root string, maxDepth uint) iter.Seq2[Entry, error] {
	return func(yield func(Entry, error) bool) {
		sep := string(filepath.Separator)

		filepath.WalkDir(root, func(path string, d fs.DirEntry, err error) error {
			if err != nil {
				if !yield(Entry{Path: path}, err) {
					return filepath.SkipAll
				}

				return nil
			}

			var depth uint

			if path != root {
				rel, _ := filepath.Rel(root, path)
				depth = uint(strings.Count(rel, sep)) + 1
			}

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
