package file

import (
	"io/fs"
	"path/filepath"
	"pls/config"
)

func ResolveIcon(info fs.FileInfo, kind string, icons *config.ListIcons) string {
	if icon, ok := icons.Names[info.Name()]; ok {
		return icon
	}

	if icon, ok := icons.Extensions[filepath.Ext(info.Name())]; ok {
		return icon
	}

	if icon, ok := icons.Kinds[kind]; ok {
		return icon
	}

	return icons.Default
}
