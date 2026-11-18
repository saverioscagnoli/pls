package main

import (
	"bufio"
	"fmt"
	"math"
	"os"
	"strings"
	"time"
)

func Find(args Args, config *Config) {
	var total int
	out := bufio.NewWriter(os.Stdout)
	start := time.Now()

	if args.Find.Depth == 0 {
		args.Find.Depth = math.MaxUint
	}

	walkoptions := WalkConcurrentOptions{
		MaxDepth: args.Find.Depth,
		All:      args.Find.All,
		Workers:  args.Find.Workers,
		Match: func(e DirEntryExt) bool {
			return strings.Contains(strings.ToLower(e.Path), strings.ToLower(args.Find.Target))
		},
	}

	for entry, err := range WalkConcurrent(args.Find.Root, walkoptions) {
		if err != nil {
			continue
		}

		out.WriteString(entry.Path)
		out.WriteByte('\n')
		total++
	}

	out.Flush()
	t := time.Since(start)

	fmt.Printf("found %d entries in %s\n", total, t)
}
