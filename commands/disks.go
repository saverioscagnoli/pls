package commands

import (
	"bufio"
	"fmt"
	"os"
	"pls/args"
	"pls/config"
	"pls/file"
	"pls/template"
	"pls/ui"
	"strings"
)

func pickThreshold(frac float64, thresholds []config.ConfigDisksBarThreshold) config.ConfigDisksBarThreshold {
	best := config.ConfigDisksBarThreshold{Min: -1, Color: ui.ColorGray, TextColor: ui.ColorWhite}

	for _, t := range thresholds {
		if frac >= t.Min && t.Min > best.Min {
			best = t
		}
	}

	return best
}

func renderBar(frac float64, config config.ConfigDisks) string {
	if frac < 0 {
		frac = 0
	} else if frac > 1 {
		frac = 1
	}

	w := bufio.NewWriter(os.Stdout)
	defer w.Flush()

	width := int(config.Bar.Width)
	t := pickThreshold(frac, config.Bar.Thresholds)
	filled := int(frac * float64(width))
	label := fmt.Sprintf("%.1f%%", frac*100)

	start := (width - len(label)) / 2
	start = max(start, 0)

	var b strings.Builder

	for i := range width {
		cell := " "

		if i >= start && i-start < len(label) {
			cell = ui.Foreground(string(label[i-start]), t.TextColor)
		}

		bg := ui.ColorGray

		if i < filled {
			bg = t.Color
		}

		b.WriteString(ui.Background(cell, bg))
	}

	b.WriteString("\x1b[0m")
	return b.String()
}

func diskTemplates(config *config.ConfigDisks) ([]*template.Template, error) {
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

func diskAlignments(config *config.ConfigDisks) []ui.Align {
	alignments := []ui.Align{}

	for _, col := range config.Columns {
		alignments = append(alignments, col.Align)
	}

	return alignments
}

func diskHeaders(config *config.ConfigDisks) []string {
	headers := []string{}

	for _, col := range config.Columns {
		if col.Header != nil {
			headers = append(headers, *col.Header)
		}
	}

	return headers
}

func diskRow(templates []*template.Template, ctx map[string]string, f *file.Filesystem, config *config.Config) ([]string, error) {
	row := []string{}

	ctx["dev"] = f.Device
	ctx["mount"] = f.Mountpoint
	ctx["fstype"] = f.Type
	ctx["bar"] = renderBar(f.Frac, config.Disks)
	ctx["total"] = ui.NewSize(f.Total).Format(config.Disks.SizeUnit)
	ctx["avail"] = ui.NewSize(f.Available).Format(config.Disks.SizeUnit)
	ctx["used"] = ui.NewSize(f.Used).Format(config.Disks.SizeUnit)

	for _, t := range templates {
		out, err := t.Format(ctx)
		if err != nil {
			return nil, fmt.Errorf("failed to format column: %v", err)
		}

		row = append(row, out)
	}

	return row, nil
}

func Disks(args args.Disks, config *config.Config) error {

	templates, err := diskTemplates(&config.Disks)
	if err != nil {
		return fmt.Errorf("invalid template: %v", err)
	}

	alignments := diskAlignments(&config.Disks)
	ctx := map[string]string{}
	rows := [][]string{}
	total := 0

	headers := diskHeaders(&config.Disks)

	if len(headers) > 0 {
		rows = append(rows, headers)
	}

	if args.Path != nil {
		f, err := file.FindFilesystem(*args.Path)
		if err != nil {
			return err
		}

		row, err := diskRow(templates, ctx, f, config)
		if err != nil {
			return err
		}

		rows = append(rows, row)
		total++
	} else {
		filesystems, err := file.AllFilesystems()
		if err != nil {
			return fmt.Errorf("failed to list filesystems: %v", err)
		}

		for _, f := range filesystems {
			if !args.All && !f.IsUser() {
				continue
			}

			row, err := diskRow(templates, ctx, &f, config)
			if err != nil {
				return err
			}

			rows = append(rows, row)
			total++
		}
	}

	if total > 0 {
		fmt.Printf("total %d\n", total)
	}

	ui.Table(rows, alignments, config.Disks.Gap)
	return nil
}
