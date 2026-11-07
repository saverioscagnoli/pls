package commands

import (
	"bufio"
	"fmt"
	"os"
	"pls/args"
	"pls/config"
	"pls/ui"
	"strings"
	"syscall"
)

func fillColor(frac float64, thresholds []config.BarThreshold) ui.Color {
	best := -1.0
	color := ui.ColorGray

	for _, t := range thresholds {
		if frac >= t.Min && t.Min > best {
			best = t.Min
			color = t.Color
		}
	}

	return color
}

func pickThreshold(frac float64, thresholds []config.BarThreshold) config.BarThreshold {
	best := config.BarThreshold{Min: -1, Color: ui.ColorGray, TextColor: ui.ColorWhite}

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

	width := int(config.BarWidth)
	t := pickThreshold(frac, config.Thresholds)
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

func Disks(args args.Disks, config *config.Config) error {
	f, err := os.Open("/proc/mounts")
	if err != nil {
		return err
	}

	defer f.Close()

	sc := bufio.NewScanner(f)

	for sc.Scan() {
		fields := strings.Fields(sc.Text())
		dev, mnt, _ := fields[0], fields[1], fields[2]

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

		_ = float64(used) / float64(used+avail)

		fmt.Printf("%s\n", renderBar(0.90, config.Disks))
	}

	if err := sc.Err(); err != nil {
		return fmt.Errorf("reading /proc/mounts: %w", err)
	}

	return nil
}
