package commands

import (
	"fmt"
	"pls/args"
	"pls/config"
	"pls/file"
	"pls/template"
	"pls/ui"
	"strconv"
	"strings"
)

func headers(config *config.List) []string {
	headers := []string{}

	for _, col := range config.Columns {
		if col.Header != nil {
			headers = append(headers, *col.Header)
		}
	}

	return headers
}

func templates(config *config.List) ([]*template.Template, error) {
	templates := []*template.Template{}

	for _, col := range config.Columns {
		t, err := template.Compile('{', '}', col.Format)

		if err != nil {
			return templates, err
		}

		templates = append(templates, t)
	}

	return templates, nil
}

func alignments(config *config.List) []ui.Align {
	alignments := []ui.Align{}

	for _, col := range config.Columns {
		alignments = append(alignments, col.Align)
	}

	return alignments
}

func List(args args.List, config config.List) error {
	skipped := 0
	total := 0
	rows := [][]string{}
	ctx := map[string]string{}

	headers := headers(&config)

	if len(headers) > 0 {
		rows = append(rows, headers)
	}

	templates, err := templates(&config)
	alignments := alignments(&config)

	if err != nil {
		return fmt.Errorf("found invalid template: %s", err)
	}

	for entry, err := range file.Walk(args.Path, args.Depth) {
		row := []string{}

		if err != nil {
			skipped++
			continue
		}

		info, err := entry.Info()
		name := info.Name()

		if strings.HasPrefix(name, ".") && !args.All {
			continue
		}

		if err != nil {
			skipped++
			continue
		}

		kind := string(entry.Kind())
		icon := file.ResolveIcon(info, kind, &config.Icons)
		permissions := info.Mode().Perm().String()[1:]
		size := strconv.FormatInt(info.Size(), 10)
		modified := info.ModTime().Format(config.ModifiedFmt)
		nlink := strconv.FormatUint(file.LinkCount(info), 10)
		owner, group, err := file.Ownership(info)

		if err != nil {
			skipped++
			continue
		}

		ctx["name"] = name
		ctx["kind"] = kind
		ctx["icon"] = icon
		ctx["permissions"] = permissions
		ctx["size"] = size
		ctx["modified"] = modified
		ctx["nlink"] = nlink
		ctx["owner"] = owner
		ctx["group"] = group

		for _, t := range templates {
			out, err := t.Format(ctx)

			if err != nil {
				return fmt.Errorf("failed to format column: %s", err)
			}

			row = append(row, out)
		}

		rows = append(rows, row)

		clear(ctx)
		total++
	}

	fmt.Printf("total %d\n", total)

	if skipped > 0 {
		fmt.Printf("skipped %d", skipped)
	}

	ui.Table(rows, alignments, config.Gap)

	return nil
}
