package args

import (
	"fmt"
	"strings"
)

func Parse(argv []string) (Args, error) {
	var all bool
	var depth *uint
	var err error
	var pos []string

loop:
	for i := 0; i < len(argv); i++ {
		arg := argv[i]

		switch {
		case arg == "--":
			pos = append(pos, argv[i+1:]...)
			break loop

		case strings.HasPrefix(arg, "--"):
			switch arg[2:] {
			case "all":
				all = true
			case "depth":
				i++

				if depth, err = parseDepth(argv, i); err != nil {
					return nil, err
				}
			default:
				return nil, fmt.Errorf("unexpected option %s", arg)
			}

		case len(arg) > 1 && arg[0] == '-':
			for _, c := range arg[1:] {
				switch c {
				case 'a':
					all = true
				case 'd':
					i++
					if depth, err = parseDepth(argv, i); err != nil {
						return nil, err
					}
				default:
					return nil, fmt.Errorf("unexpexted option -%c", c)
				}
			}

		default:
			pos = append(pos, arg)
		}
	}

	return build(pos, all, depth)
}
