package args

import (
	"errors"
	"fmt"
	"math"
	"strconv"
)

type List struct {
	Path  string
	All   bool
	Depth uint
}

type Find struct {
	Target     string // [0]
	Path       string // [1]
	All        bool   // -a --all
	Depth      uint   // -d n --depth n
	IgnoreCase bool   // -i --ignore-case
	Stats      bool   // -s --stats
	ShowErrors bool   // --show-errors
}

type Args interface {
	isArgs()
}

func (List) isArgs() {}
func (Find) isArgs() {}

func parseDepth(argv []string, i int) (*uint, error) {
	if i >= len(argv) {
		return nil, errors.New("depth: missing value")
	}

	n, err := strconv.ParseUint(argv[i], 10, 0)

	if err != nil {
		return nil, fmt.Errorf("depth: invalid value %q", argv[i])
	}

	d := uint(n)
	return &d, nil
}

func depthOr(depth *uint, def uint) uint {
	if depth != nil {
		return *depth
	}
	return def
}

func build(pos []string, all bool, depth *uint, ignoreCase bool, stats bool, showErrors bool) (Args, error) {
	var a Args
	rest := pos

	switch {
	// pls
	case len(pos) == 0:
		return List{Path: ".", All: all, Depth: depthOr(depth, 1)}, nil

	// pls find <target> ...args
	case pos[0] == "find":
		if len(pos) < 2 {
			return nil, errors.New("find: missing <target>")
		}

		path := "."

		if len(pos) >= 3 {
			path = pos[2]
		}

		a = Find{
			Target:     pos[1],
			Path:       path,
			All:        all,
			Depth:      depthOr(depth, math.MaxUint),
			IgnoreCase: ignoreCase,
			Stats:      stats,
			ShowErrors: showErrors,
		}
		rest = pos[min(3, len(pos)):]

	// pls <path> ...args
	default:
		a, rest = List{Path: pos[0], All: all, Depth: depthOr(depth, 1)}, pos[1:]
	}

	if len(rest) > 0 {
		return nil, fmt.Errorf("unexpected argument %q", rest[0])
	}

	return a, nil
}
