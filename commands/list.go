package commands

import (
	"fmt"
	"os"
	"pls/args"
	"pls/config"
	"pls/file"
	"pls/template"
	"pls/ui"
	"sort"
	"strconv"
	"strings"
)

func headers(config *config.ConfigList) []string {
	headers := []string{}

	for _, col := range config.Columns {
		if col.Header != nil {
			headers = append(headers, *col.Header)
		}
	}

	return headers
}

func templates(config *config.ConfigList) ([]*template.Template, error) {
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

func alignments(config *config.ConfigList) []ui.Align {
	alignments := []ui.Align{}

	for _, col := range config.Columns {
		alignments = append(alignments, col.Align)
	}

	return alignments
}

func List(args args.List, config *config.Config) error {
	rows := [][]string{}

	headers := headers(&config.List)

	if len(headers) > 0 {
		rows = append(rows, headers)
	}

	templates, err := templates(&config.List)

	if err != nil {
		return fmt.Errorf("found invalid template: %s", err)
	}

	alignments := alignments(&config.List)
	ctx := map[string]string{}
	fileSpecs := map[string]ui.Spec{}

	total := 0
	skipped := 0
	errCounts := map[string]int{}

	skip := func(err error) {
		skipped++
		errCounts[err.Error()]++
	}

	warned := map[string]bool{}

	warn := func(where string, style string, err error) {
		key := where + "\x00" + style

		if warned[key] {
			return
		}

		warned[key] = true
		fmt.Fprintf(os.Stderr, "%s: invalid style %q for %s: %s\n", ui.Foreground("warning", ui.ColorYellow), style, where, err)
	}

	parse := func(where string, style string) ui.Spec {
		s, err := ui.ParseSpec(style, config.Palette)
		if err != nil {
			warn(where, style, err)
			return ui.Spec{}
		}

		return s
	}

	styleSpecs := map[string]ui.Spec{}

	styleFor := func(field string, style string) ui.Spec {
		key := field + "\x00" + style

		if s, ok := styleSpecs[key]; ok {
			return s
		}

		s := parse("styles."+field, style)
		styleSpecs[key] = s
		return s
	}

	st := config.List.Styles

	for entry, err := range file.Walk(args.Path, args.Depth, args.All) {
		row := []string{}
		if err != nil {
			skip(err)
			continue
		}

		info, err := entry.Info()
		if err != nil {
			skip(err)
			continue
		}

		name := info.Name()

		kind := string(entry.Kind())

		item := file.ResolveStyle(name, kind, &config.List.Files)
		spec, ok := fileSpecs[item.Style]

		if !ok {
			spec = parse("files", item.Style)
			fileSpecs[item.Style] = spec
		}

		icon := item.Icon
		permissions := file.Permissions(info.Mode())
		size := strconv.FormatInt(info.Size(), 10)
		modified := info.ModTime().Format(config.List.ModifiedFmt)
		nlink := strconv.FormatUint(file.LinkCount(info), 10)
		owner, group, err := file.Ownership(info)

		if err != nil {
			skip(err)
			continue
		}

		ctx["name"] = styleFor("name", st.Name.For(kind)).Merge(spec).Apply(name)
		ctx["icon"] = styleFor("icon", st.Icon.For(kind)).Merge(spec).Apply(icon)
		ctx["kind"] = styleFor("kind", st.Kind.For(kind)).Apply(kind)
		ctx["permissions"] = styleFor("permissions", st.Permissions.For(kind)).Apply(permissions)
		ctx["size"] = styleFor("size", st.Size.For(kind)).Apply(size)
		ctx["modified"] = styleFor("modified", st.Modified.For(kind)).Apply(modified)
		ctx["nlink"] = styleFor("nlink", st.NLink.For(kind)).Apply(nlink)
		ctx["owner"] = styleFor("owner", st.Owner.For(kind)).Apply(owner)
		ctx["group"] = styleFor("group", st.Group.For(kind)).Apply(group)
		ctx["depth"] = strconv.FormatUint(uint64(entry.Depth-1), 10)

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

	if total > 0 {
		fmt.Printf("total %d\n", total)
	}

	ui.Table(rows, alignments, config.List.Gap)

	if skipped > 0 {
		parts := make([]string, 0, len(errCounts))
		for msg, n := range errCounts {
			parts = append(parts, fmt.Sprintf("%d %s", n, msg))
		}

		// map order is random, keep output stable
		sort.Strings(parts)
		fmt.Fprintf(os.Stderr, "%d skipped: %s\n", skipped, ui.Foreground(strings.Join(parts, ", "), ui.ColorRed))
	}

	return nil
}
