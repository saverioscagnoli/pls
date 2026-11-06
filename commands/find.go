package commands

import (
	"bufio"
	"context"
	"fmt"
	"os"
	"path/filepath"
	"pls/args"
	"pls/config"
	"pls/file"
	"pls/ui"
	"strings"
	"time"
)

func indexFoldASCII(s, sub string) int {
	n := len(sub)

	for i := 0; i+n <= len(s); i++ {
		j := 0

		for j < n {
			a := s[i+j]

			if 'A' <= a && a <= 'Z' {
				a += 'a' - 'A'
			}

			if a != sub[j] {
				break
			}

			j++
		}

		if j == n {
			return i
		}
	}

	return -1
}

func nameIndexer(target string, ignoreCase bool) func(string) int {
	if ignoreCase {
		t := strings.ToLower(target)
		return func(name string) int { return indexFoldASCII(name, t) }
	}

	return func(name string) int { return strings.Index(name, target) }
}

func Find(args args.Find, config config.ConfigFind) {
	t := time.Now()
	count := 0

	root, err := filepath.Abs(args.Path)
	if err != nil {
		fmt.Println(err)
		return
	}

	w := bufio.NewWriterSize(os.Stdout, 64*1024)
	defer w.Flush()

	index := nameIndexer(args.Target, args.IgnoreCase)
	n := len(args.Target)

	dim := ui.ColorGray.FG()
	blue := ui.ColorBlue.FG()
	hl := "\033[1m" + ui.ColorYellow.FG()

	for e, err := range file.Search(context.Background(), root, file.SearchOptions{
		MaxDepth: args.Depth,
		All:      args.All,
		Match:    func(e file.Entry) bool { return index(e.Name()) >= 0 },
	}) {
		if err != nil {
			continue
		}

		name := e.Name()
		dir := e.Path[:len(e.Path)-len(name)]
		i := index(name)

		nameColor := ""

		if e.IsDir() {
			nameColor = blue
		}

		w.WriteString(dim)
		w.WriteString(dir)
		w.WriteString(ui.Reset)
		w.WriteString(nameColor)
		w.WriteString(name[:i])
		w.WriteString(hl)
		w.WriteString(name[i : i+n])
		w.WriteString(ui.Reset)
		w.WriteString(nameColor)
		w.WriteString(name[i+n:])

		if e.IsDir() {
			w.WriteByte('/')
		}

		w.WriteString(ui.Reset)
		w.WriteByte('\n')
		count++
	}

	w.Flush()
	fmt.Fprintf(os.Stderr, "found %d entries in %s\n", count, time.Since(t))
}
