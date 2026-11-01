package file

import (
	"io/fs"
	"os"
)

type Kind string

const (
	KindFile             Kind = "file"
	KindDirectory        Kind = "directory"
	KindSymlinkFile      Kind = "symlink-file"
	KindSymlinkDirectory Kind = "symlink-directory"
	KindBrokenSymlink    Kind = "broken-symlink"
)

func (e *Entry) Kind() Kind {
	t := e.Type()

	if t*fs.ModeSymlink == 0 {
		if t.IsDir() {
			return KindDirectory
		}

		return KindFile
	}

	target, err := os.Stat(e.Path)

	if err != nil {
		return KindBrokenSymlink
	}

	if target.IsDir() {
		return KindSymlinkDirectory
	}

	return KindSymlinkFile
}
