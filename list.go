package main

import (
	"fmt"
	"os"
	"strconv"
)

func List(args Args, config *Config) {
	var rows [][]string
	var errors []error
	var total int

	aligns := ConfigAligns(config.List.Columns)
	templates, errors := ConfigTemplates(config.List.Columns)
	ctx := make(map[string]string)

	if len(errors) > 0 {
		for _, err := range errors {
			fmt.Fprintf(os.Stderr, "template: %s", err)
		}
		return
	}

	for entry := range Walk(args.List.Path, args.List.Depth, args.List.All) {
		var row []string

		info, err := entry.Info()
		if err != nil {
			errors = append(errors, err)
			continue
		}

		owner, group, err := DirEntryOwnership(info)
		if err != nil {
			errors = append(errors, err)
			continue
		}

		ctx["name"] = info.Name()
		ctx["depth"] = strconv.FormatUint(uint64(entry.Depth-1), 10)
		ctx["kind"] = string(entry.Kind())
		ctx["permissions"] = DirEntryPermissions(info.Mode())
		ctx["size"] = NewSize(info.Size()).Format(config.List.SizeFmt)
		ctx["modified"] = info.ModTime().Format(config.List.ModifiedFmt)
		ctx["nlink"] = strconv.FormatUint(DirEntryNLink(info), 10)
		ctx["owner"] = owner
		ctx["group"] = group

		for _, t := range templates {
			s, err := t.Format(ctx)
			if err != nil {
				errors = append(errors, err)
				continue
			}

			row = append(row, s)
		}

		total++
		rows = append(rows, row)
		clear(ctx)
	}

	if total > 0 {
		fmt.Printf("total %d\n", total)
	}

	Table(rows, aligns, config.List.Gap)
}
