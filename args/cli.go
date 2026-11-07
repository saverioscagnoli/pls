package args

type List struct {
	Path  string `arg:"" default:"."`
	All   bool   `short:"a"`
	Depth uint   `short:"d" default:"1"`
}

type Find struct {
	Target     string `arg:""`
	Path       string `arg:"" default:"."`
	All        bool   `short:"a"`
	IgnoreCase bool   `short:"i"`
	Stats      bool   `short:"s"`
	ShowErrors bool   `name:"show-errors"`
	Depth      uint   `short:"d" default:"0"`
}

type Disks struct {
}

type Cli struct {
	List  List   `cmd:"" default:"withargs" help:"List directory contents."`
	Find  Find   `cmd:"" help:"Find files by name."`
	Disks Disks  `cmd:"" help:"List every disk"`
	Color string `enum:"auto,always,never" default:"auto"`
}
