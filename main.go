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
		if err := commands.List(a, &config.Palette, config.List); err != nil {
			fmt.Fprintln(os.Stderr, err)
			os.Exit(1)
		}
	case args.Find:
		commands.Find(a, config.Find)
	}
}
