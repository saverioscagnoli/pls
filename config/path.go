package config

import (
	"fmt"
	"os"
	"path/filepath"
)

func Dir() string {
	if dir, err := os.UserConfigDir(); err == nil {
		return filepath.Join(dir, "pls")
	}

	fmt.Fprintln(os.Stderr, "both $XDG_CONFIG_HOME and $HOME are not set. Falling back to /tmp/pls")
	return filepath.Join("/tmp", "pls")
}

func File() string {
	return filepath.Join(Dir(), "config.json")
}
