package main

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

type DirEntryExt struct {
	Path  string
	Depth uint
	fs.DirEntry
}

type WalkOptions struct {
	MaxDepth uint
	All      bool
	SortFunc func(fs.DirEntry, fs.DirEntry) int
}

func walkInner(root string, depth uint, options WalkOptions, yield func(DirEntryExt, error) bool) bool {
	entries, err := os.ReadDir(root)

	if err != nil {
		return yield(DirEntryExt{Path: root, Depth: depth}, err)
	}

	if options.SortFunc != nil {
		slices.SortStableFunc(entries, options.SortFunc)
	}

	for _, entry := range entries {
		path := filepath.Join(root, entry.Name())

		if !options.All && entry.Name()[0] == '.' {
			continue
		}

		if !yield(DirEntryExt{Path: path, Depth: depth, DirEntry: entry}, nil) {
			return false
		}

		if entry.IsDir() && depth < options.MaxDepth {
			if !walkInner(path, depth+1, options, yield) {
				return false
			}
		}
	}

	return true
}

func Walk(root string, o WalkOptions) iter.Seq2[DirEntryExt, error] {
	return func(yield func(DirEntryExt, error) bool) {
		info, err := os.Lstat(root)
		if err != nil {
			yield(DirEntryExt{Path: root}, err)
			return
		}

		if target, err := os.Stat(root); err == nil && target.IsDir() {
			walkInner(root, 1, o, yield)
			return
		}

		yield(DirEntryExt{Path: root, Depth: 1, DirEntry: fs.FileInfoToDirEntry(info)}, nil)
	}
}

type WalkConcurrentOptions struct {
	MaxDepth uint
	All      bool
	Workers  uint
	Match    func(DirEntryExt) bool
	Prune    func(DirEntryExt) bool
}

type batch struct {
	entries []DirEntryExt
	err     error
	errAt   DirEntryExt
}

func WalkConcurrent(root string, o WalkConcurrentOptions) iter.Seq2[DirEntryExt, error] {
	return func(yield func(DirEntryExt, error) bool) {
		info, err := os.Stat(root)
		if err != nil {
			yield(DirEntryExt{Path: root}, err)
			return
		}

		if !info.IsDir() {
			e := DirEntryExt{Path: root, Depth: 1, DirEntry: fs.FileInfoToDirEntry(info)}

			if o.Match == nil || o.Match(e) {
				yield(e, nil)
			}
			return
		}

		workers := o.Workers
		if workers == 0 {
			workers = uint(runtime.GOMAXPROCS(0))
		}

		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()

		out := make(chan batch, workers)
		sem := make(chan struct{}, workers)
		var wg sync.WaitGroup

		send := func(b batch) bool {
			select {
			case out <- b:
				return true
			case <-ctx.Done():
				return false
			}
		}

		type directory struct {
			path  string
			depth uint
		}

		scan := func(dir string, depth uint) []string {
			var hits []DirEntryExt
			var subdirs []string
			var errs []batch

			prefix := dir
			if prefix != "/" {
				prefix += "/"
			}

			err := ReadDirent(dir, func(name string, typ fs.FileMode, unknown bool) {
				if !o.All && name[0] == '.' {
					return
				}

				path := prefix + name
				de := rawEntry{path: path, name: name, typ: typ}

				if unknown {
					fi, err := os.Lstat(path)
					if err != nil {
						errs = append(errs, batch{err: err, errAt: DirEntryExt{Path: path, Depth: depth}})
						return
					}

					de.typ = fi.Mode().Type()
				}

				e := DirEntryExt{Path: path, Depth: depth, DirEntry: de}

				if o.Match == nil || o.Match(e) {
					hits = append(hits, e)
				}

				if de.IsDir() && depth < o.MaxDepth && (o.Prune == nil || !o.Prune(e)) {
					subdirs = append(subdirs, path)
				}
			})

			if err != nil {
				send(batch{err: err, errAt: DirEntryExt{Path: dir, Depth: depth}})
				return nil
			}

			for _, b := range errs {
				if !send(b) {
					return nil
				}
			}

			if len(hits) > 0 && !send(batch{entries: hits}) {
				return nil
			}

			return subdirs
		}

		var visit func(directory)
		visit = func(current directory) {
			defer wg.Done()
			defer func() { <-sem }()
			var pending []directory
			for ctx.Err() == nil {
				for _, path := range scan(current.path, current.depth) {
					if ctx.Err() != nil {
						return
					}
					child := directory{path, current.depth + 1}
					select {
					case sem <- struct{}{}:
						wg.Add(1)
						go visit(child)
					default:
						pending = append(pending, child)
					}
				}
				if len(pending) == 0 {
					return
				}
				last := len(pending) - 1
				current = pending[last]
				pending[last] = directory{}
				pending = pending[:last]
			}
		}

		sem <- struct{}{}
		wg.Add(1)
		go visit(directory{root, 1})
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
