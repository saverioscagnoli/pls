package commands

import (
	"fmt"
	"os"
	"pls/args"
	"pls/config"
	"pls/ui"
	"pls/walk"
	"strconv"
	"syscall"
)

func linkCount(info os.FileInfo) uint64 {
	if st, ok := info.Sys().(*syscall.Stat_t); ok {
		return uint64(st.Nlink)
	}

	return 1
}

func List(args args.List, config config.List) {
	skipped := 0
	total := 0
	rows := [][]string{}

	for entry, err := range walk.Dir(args.Path, args.Depth) {
		row := []string{}

		if err != nil {
			skipped++
			continue
		}

		info, err := entry.Info()
		var t string

		if err != nil {
			skipped++
			continue
		}

		if info.IsDir() {
			t = "dir"
		} else {
			t = "file"
		}

		row = append(row, t)
		row = append(row, ui.Foreground(entry.Name(), ui.ColorCyan))
		row = append(row, strconv.FormatInt(info.Size(), 10))
		row = append(row, info.ModTime().Format("Jan _2 15:04"))
		row = append(row, strconv.FormatUint(linkCount(info), 10))

		rows = append(rows, row)

		total++
	}

	if skipped > 0 {
		fmt.Printf("%d entries were skipped\n", skipped)
	}

	fmt.Printf("total %d\n", total)
	ui.Table(rows, []ui.Align{ui.AlignLeft, ui.AlignLeft, ui.AlignLeft}, 2)
}
