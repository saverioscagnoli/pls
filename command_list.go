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

		styles := config.List.Styles
		vars := config.List.VarStyles

		depth := strconv.FormatUint(uint64(entry.Depth-1), 10)
		name := info.Name()
		kind := entry.Kind()
		permissions := DirEntryPermissions(info.Mode())
		size := NewSize(info.Size()).Format(config.List.SizeFmt)
		modified := info.ModTime().Format(config.List.ModifiedFmt)
		nlink := strconv.FormatUint(DirEntryNLink(info), 10)
		owner, group, err := DirEntryOwnership(info)
		if err != nil {
			errors = append(errors, err)
			continue
		}

		item := styles.Resolve(name, kind)
		// Depth is also used as a numeric template repeat count.
		ctx["depth"] = depth
		ctx["name"] = ApplyColor(styles.ColorFor(name, kind, &vars.Name), name)
		ctx["icon"] = ApplyColor(styles.ColorFor(name, kind, &vars.Icon), item.Icon)
		ctx["kind"] = vars.Kind.Apply(kind, string(kind))
		ctx["permissions"] = vars.Permissions.Apply(kind, permissions)
		ctx["size"] = vars.Size.Apply(kind, size)
		ctx["modified"] = vars.Modified.Apply(kind, modified)
		ctx["nlink"] = vars.NLink.Apply(kind, nlink)
		ctx["owner"] = vars.Owner.Apply(kind, owner)
		ctx["group"] = vars.Group.Apply(kind, group)

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

	if len(errors) > 0 {
		fmt.Printf("%d errors\n", len(errors))
	}

	for _, err := range errors {
		fmt.Fprintln(os.Stderr, err)
	}
}
