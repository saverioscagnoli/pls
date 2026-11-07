package main

import (
	"fmt"
	"os"
	"pls/args"
	"pls/commands"
	"pls/config"

	"github.com/alecthomas/kong"
)

func main() {
	var cli args.Cli
	parser, err := kong.New(&cli,
		kong.Name("pls"),
		kong.Description("super description bruh"),
		kong.ConfigureHelp(kong.HelpOptions{Compact: true}),
	)

	if err != nil {
		panic(err)
	}

	ctx, err := parser.Parse(os.Args[1:])
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

	switch ctx.Selected().Name {
	case "list":
		commands.List(cli.List, config)
	case "find":
		commands.Find(cli.Find, config)
	case "disks":
		commands.Disks(cli.Disks, config)
	}

}
