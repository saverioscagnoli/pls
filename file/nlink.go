package file

import (
	"fmt"
	"io/fs"
	"os/user"
	"strconv"
	"syscall"
)

func LinkCount(info fs.FileInfo) uint64 {
	if st, ok := info.Sys().(*syscall.Stat_t); ok {
		return uint64(st.Nlink)
	}

	return 1
}

func Ownership(info fs.FileInfo) (string, string, error) {
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
