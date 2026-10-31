package ui

import (
	"fmt"
	"regexp"
	"strings"
	"unicode/utf8"
)

type Align int

const (
	AlignLeft   Align = 0
	AlignRight  Align = 1
	AlignCenter Align = 2
)

var ansiRe = regexp.MustCompile(`\x1b\[[0-9;]*m`)

func visibleLen(s string) int {
	return utf8.RuneCountInString(ansiRe.ReplaceAllString(s, ""))
}

func pad(s string, width int, align Align) string {
	gap := width - visibleLen(s)

	if gap <= 0 {
		return s
	}

	switch align {
	case AlignRight:
		return strings.Repeat(" ", gap) + s
	case AlignCenter:
		l := gap / 2
		return strings.Repeat(" ", l) + s + strings.Repeat(" ", gap-l)
	default:
		return s + strings.Repeat(" ", gap)
	}
}

func Table(rows [][]string, aligns []Align, gap int) {
	widths := []int{}

	for _, row := range rows {
		for i, cell := range row {
			n := visibleLen(cell)

			if i >= len(widths) {
				widths = append(widths, n)
			} else if n > widths[i] {
				widths[i] = n
			}
		}
	}

	sep := strings.Repeat(" ", gap)

	for _, row := range rows {
		cells := make([]string, len(row))

		for i, cell := range row {
			a := AlignLeft

			if i < len(aligns) {
				a = aligns[i]
			}

			cells[i] = pad(cell, widths[i], a)
		}

		fmt.Println(strings.TrimRight(strings.Join(cells, sep), " "))
	}
}
