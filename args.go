package main

type ArgsList struct {
	Path  string `arg:"" default:"."`
	All   bool   `short:"a"`
	Depth uint   `short:"d" default:"1"`
}

type ArgsFind struct {
	Target  string `arg:""`
	Root    string `arg:"" default:"."`
	All     bool   `short:"a"`
	Depth   uint   `short:"d" default:"0"`
	Workers uint   `short:"w" default:"0"`
}

type Args struct {
	Config  *string  `short:"c"`
	NoColor bool     `name:"no-color"`
	List    ArgsList `cmd:"" default:"withargs"`
	Find    ArgsFind `cmd:""`
}
