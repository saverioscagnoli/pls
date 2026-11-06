package file

import (
	"io/fs"
	"os"
	"sync"
	"syscall"
	"unsafe"
)

var direntBufPool = sync.Pool{New: func() any { b := make([]byte, 64*1024); return &b }}

type rawEntry struct {
	path string
	name string
	typ  fs.FileMode
}

func (e rawEntry) Name() string               { return e.name }
func (e rawEntry) IsDir() bool                { return e.typ == fs.ModeDir }
func (e rawEntry) Type() fs.FileMode          { return e.typ }
func (e rawEntry) Info() (fs.FileInfo, error) { return os.Lstat(e.path) }

func dtToMode(t uint8) fs.FileMode {
	switch t {
	case syscall.DT_DIR:
		return fs.ModeDir
	case syscall.DT_LNK:
		return fs.ModeSymlink
	case syscall.DT_FIFO:
		return fs.ModeNamedPipe
	case syscall.DT_SOCK:
		return fs.ModeSocket
	case syscall.DT_CHR:
		return fs.ModeDevice | fs.ModeCharDevice
	case syscall.DT_BLK:
		return fs.ModeDevice
	}

	return 0
}

func readdirFast(dir string, fn func(name string, typ fs.FileMode, unknown bool)) error {
	fd, err := syscall.Open(dir, syscall.O_RDONLY|syscall.O_DIRECTORY|syscall.O_CLOEXEC, 0)
	if err != nil {
		return &fs.PathError{Op: "open", Path: dir, Err: err}
	}

	defer syscall.Close(fd)

	bp := direntBufPool.Get().(*[]byte)
	defer direntBufPool.Put(bp)

	buf := *bp

	for {
		n, err := syscall.ReadDirent(fd, buf)
		if err == syscall.EINTR {
			continue
		}

		if err != nil {
			return &fs.PathError{Op: "readdirent", Path: dir, Err: err}
		}

		if n <= 0 {
			return nil
		}

		for off := 0; off < n; {
			d := (*syscall.Dirent)(unsafe.Pointer(&buf[off]))
			off += int(d.Reclen)

			nb := (*[256]byte)(unsafe.Pointer(&d.Name[0]))
			l := 0

			for l < len(nb) && nb[l] != 0 {
				l++
			}

			if d.Ino == 0 || (l == 1 && nb[0] == '.') || (l == 2 && nb[0] == '.' && nb[1] == '.') {
				continue
			}

			fn(string(nb[:l]), dtToMode(d.Type), d.Type == syscall.DT_UNKNOWN)
		}
	}
}
