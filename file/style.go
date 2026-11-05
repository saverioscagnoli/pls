package file

import (
	"path/filepath"
	"pls/config"
)

func ResolveStyle(name string, kind string, config *config.ConfigListFileStyles) config.ConfigListStyleItem {
	if item, ok := config.Names[name]; ok {
		return item
	}

	ext := filepath.Ext(name)
	if item, ok := config.Extensions[ext]; ok {
		return item
	}

	if item, ok := config.Kinds[kind]; ok {
		return item
	}

	return config.Default
}
