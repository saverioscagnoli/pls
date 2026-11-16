package main

type ArgsList struct {
	Path  string `arg:"" default:"."`
	All   bool   `short:"a"`
	Depth uint   `short:"d" default:"1"`
}

type ArgsFind struct {
	Target string
}

type Args struct {
	Config  *string  `short:"c"`
	NoColor bool     `name:"no-color"`
	List    ArgsList `cmd:"" default:"withargs"`
}
