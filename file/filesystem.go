package file

import (
	"fmt"
	"path/filepath"
	"strings"
	"syscall"

	"github.com/moby/sys/mountinfo"
)

type Filesystem struct {
	Mountpoint string
	Device     string
	Type       string
	Total      int64
	Available  int64
	Used       int64
	Frac       float64
}

func AllFilesystems() ([]Filesystem, error) {
	mounts, err := mountinfo.GetMounts(nil)
	if err != nil {
		return nil, err
	}

	f := make([]Filesystem, 0, len(mounts))

	for _, m := range mounts {
		var st syscall.Statfs_t
		if err := syscall.Statfs(m.Mountpoint, &st); err != nil {
			return f, err
		}

		total := int64(st.Blocks) * st.Bsize
		used := (int64(st.Blocks) - int64(st.Bfree)) * st.Bsize
		avail := int64(st.Bavail) * st.Bsize

		var frac float64
		if total > 0 {
			frac = float64(used) / float64(total)
		}

		f = append(f, Filesystem{
			Mountpoint: m.Mountpoint,
			Device:     m.Source,
			Type:       m.FSType,
			Total:      total,
			Available:  avail,
			Used:       used,
			Frac:       frac,
		})
	}

	return f, nil
}

func FindFilesystem(path string) (*Filesystem, error) {
	path, err := filepath.Abs(path)
	if err != nil {
		return nil, err
	}

	path, err = filepath.EvalSymlinks(path)
	if err != nil {
		return nil, err
	}

	mounts, err := mountinfo.GetMounts(nil)
	if err != nil {
		return nil, err
	}

	var best *mountinfo.Info

	for _, m := range mounts {
		rel, err := filepath.Rel(m.Mountpoint, path)
		if err != nil || rel == ".." || strings.HasPrefix(rel, ".."+string(filepath.Separator)) {
			continue
		}

		if best == nil || len(m.Mountpoint) > len(best.Mountpoint) {
			best = m
		}
	}

	if best == nil {
		return nil, fmt.Errorf("no filesystem was found for %q", path)
	}

	var st syscall.Statfs_t
	if err := syscall.Statfs(path, &st); err != nil {
		return nil, err
	}

	total := int64(st.Blocks) * st.Bsize
	used := (int64(st.Blocks) - int64(st.Bfree)) * st.Bsize
	avail := int64(st.Bavail) * st.Bsize

	var frac float64
	if total > 0 {
		frac = float64(used) / float64(total)
	}

	return &Filesystem{
		Mountpoint: best.Mountpoint,
		Device:     best.Source,
		Type:       best.FSType,
		Total:      total,
		Used:       used,
		Available:  avail,
		Frac:       frac,
	}, nil
}

func (f *Filesystem) IsUser() bool {
	if strings.HasPrefix(f.Device, "/dev/") {
		return true
	}

	switch f.Type {
	case "nfs", "nfs4", "cifs", "fuse.sshfs":
		return true
	default:
		return false
	}
}
