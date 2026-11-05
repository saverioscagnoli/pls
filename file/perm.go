package file

import "io/fs"

func Permissions(m fs.FileMode) string {
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
