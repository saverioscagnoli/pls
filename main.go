package main

import (
	"fmt"
	"os"
	"pls/args"
	"pls/commands"
	"pls/config"
)

func main() {
	a, err := args.Parse(os.Args[1:])

	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(2)
	}

	path := config.File()
	config, err := config.Parse(path)

	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}

	switch a := a.(type) {
	case args.List:
		commands.List(a, config.List)
	case args.Find:
		commands.Find(a, config.Find)
	}
}
