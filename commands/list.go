package commands

import (
	"fmt"
	"pls/args"
	"pls/config"
	"pls/file"
	"pls/ui"
	"strconv"
)

func List(args args.List, config config.List) {
	skipped := 0
	total := 0
	rows := [][]string{}
	alignments := []ui.Align{}
	ctx := map[string]string{}

	for entry, err := range file.Walk(args.Path, args.Depth) {
		row := []string{}

		if err != nil {
			skipped++
			continue
		}

		info, err := entry.Info()

		if err != nil {
			skipped++
			continue
		}

		ctx["kind"] = string(entry.Kind())
		ctx["name"] = entry.Name()
		ctx["permissions"] = info.Mode().Perm().String()[1:]
		ctx["size"] = strconv.FormatInt(info.Size(), 10)
		ctx["modified"] = info.ModTime().Format(config.ModifiedFmt)
		ctx["nlink"] = strconv.FormatUint(file.LinkCount(info), 10)

		rows = append(rows, row)

		clear(ctx)
		total++
	}

	fmt.Printf("total %d\n", total)

	if skipped > 0 {
		fmt.Printf("skipped %d", skipped)
	}

	ui.Table(rows, alignments, config.Gap)
}
