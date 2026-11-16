package main

import (
	"fmt"
	"os"

	"github.com/alecthomas/kong"
	"github.com/fatih/color"
)

func main() {
	var args Args
	parser, err := kong.New(&args,
		kong.Name("pls"),
		kong.Description("super mega detailed description"),
		kong.ConfigureHelp(kong.HelpOptions{Compact: true}),
	)

	if err != nil {
		panic(err)
	}

	ctx, err := parser.Parse(os.Args[1:])
	if err != nil {
		panic(err)
	}

	if args.NoColor {
		color.NoColor = true
	}

	config, err := ConfigParse(args.Config)
	if err != nil {
		panic(err)
	}

	switch ctx.Selected().Name {
	case "list":
		List(args, config)
	case "find":
		fmt.Println("find")
	}
}
