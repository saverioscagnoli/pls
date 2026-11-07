package commands

import (
	"bufio"
	"fmt"
	"os"
	"pls/args"
	"pls/config"
	"pls/template"
	"pls/ui"
	"strings"
	"syscall"
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

func Disks(args args.Disks, config *config.Config) error {
	f, err := os.Open("/proc/mounts")
	if err != nil {
		return err
	}

	defer f.Close()

	sc := bufio.NewScanner(f)

	templates, err := diskTemplates(&config.Disks)

	if err != nil {
		return fmt.Errorf("found invalid template: %s", err)
	}

	ctx := map[string]string{}
	rows := [][]string{}

	for sc.Scan() {
		row := []string{}
		fields := strings.Fields(sc.Text())
		dev, mnt, fstype := fields[0], fields[1], fields[2]

		if !strings.HasPrefix(dev, "/dev/") {
			continue
		}

		var st syscall.Statfs_t
		if err := syscall.Statfs(mnt, &st); err != nil {
			continue
		}

		bsize := uint64(st.Bsize)
		total := st.Blocks * bsize
		avail := st.Bavail * bsize
		used := (st.Blocks - st.Bfree) * bsize

		if total == 0 {
			continue
		}

		frac := float64(used) / float64(used+avail)

		ctx["dev"] = dev
		ctx["mount"] = mnt
		ctx["fstype"] = fstype
		ctx["bar"] = renderBar(frac, config.Disks)
		ctx["total"] = string(total)
		ctx["avail"] = string(avail)
		ctx["used"] = string(used)

		for _, t := range templates {
			out, err := t.Format(ctx)

			if err != nil {
				return fmt.Errorf("failed to format column: %s", err)
			}

			row = append(row, out)
		}

		rows = append(rows, row)
	}

	if err := sc.Err(); err != nil {
		return fmt.Errorf("reading /proc/mounts: %w", err)
	}

	ui.Table(rows, []ui.Align{}, 2)

	return nil
}
