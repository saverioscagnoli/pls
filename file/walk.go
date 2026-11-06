package file

import (
	"context"
	"io/fs"
	"iter"
	"os"
	"path/filepath"
	"runtime"
	"slices"
	"sync"
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

func readDirUnsorted(dir string) ([]os.DirEntry, error) {
	f, err := os.Open(dir)
	if err != nil {
		return nil, err
	}

	defer f.Close()
	return f.ReadDir(-1)
}

type SearchOptions struct {
	MaxDepth uint
	All      bool
	Workers  int
	Match    func(Entry) bool
	Prune    func(Entry) bool
}

type batch struct {
	entries []Entry
	err     error
	errAt   Entry
}

func Search(ctx context.Context, root string, o SearchOptions) iter.Seq2[Entry, error] {
	return func(yield func(Entry, error) bool) {
		if o.Workers < 1 {
			o.Workers = runtime.GOMAXPROCS(0)
		}

		ctx, cancel := context.WithCancel(ctx)
		defer cancel()

		out := make(chan batch, o.Workers*4)
		sem := make(chan struct{}, o.Workers)
		var wg sync.WaitGroup

		send := func(b batch) bool {
			select {
			case out <- b:
				return true
			case <-ctx.Done():
				return false
			}
		}

		var visit func(dir string, depth uint)
		visit = func(dir string, depth uint) {
			defer wg.Done()

			select {
			case sem <- struct{}{}:
			case <-ctx.Done():
				return
			}

			entries, err := readDirUnsorted(dir)
			<-sem

			if err != nil {
				send(batch{err: err, errAt: Entry{Path: dir, Depth: depth}})
				return
			}

			var hits []Entry
			var subdirs []string

			for _, de := range entries {
				name := de.Name()
				if !o.All && name[0] == '.' {
					continue
				}

				e := Entry{Path: filepath.Join(dir, name), Depth: depth, DirEntry: de}

				if o.Match == nil || o.Match(e) {
					hits = append(hits, e)
				}

				if de.IsDir() && depth < o.MaxDepth && (o.Prune == nil || !o.Prune(e)) {
					subdirs = append(subdirs, e.Path)
				}
			}

			if len(hits) > 0 && !send(batch{entries: hits}) {
				return
			}

			for _, p := range subdirs {
				wg.Add(1)
				go visit(p, depth+1)
			}
		}

		info, err := os.Stat(root)
		if err != nil {
			yield(Entry{Path: root}, err)
			return
		}

		if !info.IsDir() {
			yield(Entry{Path: root, Depth: 1, DirEntry: fs.FileInfoToDirEntry(info)}, nil)
			return
		}

		wg.Add(1)
		go visit(root, 1)
		go func() { wg.Wait(); close(out) }()

	loop:
		for b := range out {
			if b.err != nil {
				if !yield(b.errAt, b.err) {
					break
				}

				continue
			}

			for _, e := range b.entries {
				if !yield(e, nil) {
					break loop
				}
			}
		}

		cancel()
		for range out {
		}
	}

}
