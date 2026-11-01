package file

import (
	"io/fs"
	"syscall"
)

func LinkCount(info fs.FileInfo) uint64 {
	if st, ok := info.Sys().(*syscall.Stat_t); ok {
		return uint64(st.Nlink)
	}

	return 1
}
