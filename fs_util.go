package main

import (
	"fmt"
	"io/fs"
	"os/user"
	"strconv"
	"syscall"
)

func DirEntryPermissions(m fs.FileMode) string {
	b := []byte(m.Perm().String()[1:])

	special := func(i int, set bool, lower, upper byte) {
		if !set {
			return
		}
		if b[i] == 'x' {
			b[i] = lower
		} else {
			b[i] = upper
		}
	}

	special(2, m&fs.ModeSetuid != 0, 's', 'S')
	special(5, m&fs.ModeSetgid != 0, 's', 'S')
	special(8, m&fs.ModeSticky != 0, 't', 'T')

	return string(b)
}

func DirEntryNLink(info fs.FileInfo) uint64 {
	if st, ok := info.Sys().(*syscall.Stat_t); ok {
		return uint64(st.Nlink)
	}

	return 1
}

func DirEntryOwnership(info fs.FileInfo) (string, string, error) {
	st, ok := info.Sys().(*syscall.Stat_t)

	if !ok {
		return "", "", fmt.Errorf("not a unix file")
	}

	uid := strconv.FormatUint(uint64(st.Uid), 10)
	gid := strconv.FormatUint(uint64(st.Gid), 10)

	owner, err := user.LookupId(uid)

	if err != nil {
		return "", "", fmt.Errorf("could not retrieve owner")
	}

	group, err := user.LookupGroupId(gid)

	if err != nil {
		return "", "", fmt.Errorf("could not retrieve owner group")
	}

	return owner.Username, group.Name, nil
}
