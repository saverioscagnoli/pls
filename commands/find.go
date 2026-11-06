package commands

import (
	"context"
	"fmt"
	"path/filepath"
	"pls/args"
	"pls/config"
	"pls/file"
	"strings"
	"time"
)

func nameMatcher(target string, ignoreCase bool) func(file.Entry) bool {
	if ignoreCase {
		t := strings.ToLower(target)
		return func(e file.Entry) bool {
			return strings.Contains(strings.ToLower(e.Name()), t)
		}
	}

	return func(e file.Entry) bool {
		return strings.Contains(e.Name(), target)
	}
}

func Find(args args.Find, config config.ConfigFind) {
	t := time.Now()
	count := 0

	root, err := filepath.Abs(args.Path)
	if err != nil {
		fmt.Println(err)
		return
	}

	for e, err := range file.Search(context.Background(), root, file.SearchOptions{
		MaxDepth: args.Depth,
		All:      args.All,
		Match:    nameMatcher(args.Target, args.IgnoreCase),
	}) {
		if err != nil {
			continue
		}

		fmt.Println(e.Path)
		count++
	}

	fmt.Printf("found %d entries in %s\n", count, time.Since(t))
}
