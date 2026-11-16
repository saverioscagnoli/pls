package main

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"fmt"
	"os"
	"path/filepath"

	"github.com/fatih/color"
)

type ConfigColumn struct {
	Header *string `json:"header"`
	Format string  `json:"format"`
	Align  Align   `json:"align"`
}

type ConfigList struct {
	Columns     []ConfigColumn `json:"columns"`
	Gap         uint           `json:"gap"`
	ModifiedFmt string         `json:"modified-fmt"`
}

type ConfigFind struct{}

type Config struct {
	Palette map[string]color.Color `json:"palette"`
	List    ConfigList             `json:"list"`
	Find    ConfigFind             `json:"find"`
}

func ConfigDefault() Config {
	return Config{
		Palette: map[string]color.Color{},
		List: ConfigList{
			Columns: []ConfigColumn{
				{Format: "{name}"},
			},
			Gap:         2,
			ModifiedFmt: "Jan 02 15:04",
		},
		Find: ConfigFind{},
	}
}

func ConfigDir() string {
	if dir, err := os.UserConfigDir(); err == nil {
		return filepath.Join(dir, "pls")
	}

	fmt.Fprintln(os.Stderr, "both $XDG_CONFIG_HOME and $HOME are not set. Falling back to /tmp/pls")
	return filepath.Join("/tmp", "pls")
}

func ConfigFile() string {
	return filepath.Join(ConfigDir(), "config.json")
}

func ConfigSave(path string, config *Config) error {
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return err
	}

	data, err := json.Marshal(config, jsontext.WithIndent("  "))
	if err != nil {
		return err
	}

	return os.WriteFile(path, append(data, '\n'), 0o644)
}

func ConfigParse(customPath *string) (*Config, error) {
	var path string

	if customPath != nil {
		if *customPath == "" {
			path = ConfigFile()
		} else {
			path = *customPath
		}
	} else {
		path = ConfigFile()
	}

	config := ConfigDefault()
	bytes, err := os.ReadFile(path)

	// If it doesn't exist, try to save the default config and use that
	if os.IsNotExist(err) {
		if err := ConfigSave(path, &config); err != nil {
			return nil, err
		}

		return &config, nil
	}

	if err != nil {
		return nil, err
	}

	if err := json.Unmarshal(bytes, &config,
		json.RejectUnknownMembers(true),
		json.WithUnmarshalers(json.UnmarshalFunc(DecodeColor)),
	); err != nil {
		return nil, fmt.Errorf("decode config %q: %w", path, err)
	}

	return &config, nil
}

func ConfigTemplates(columns []ConfigColumn) ([]*Template, []error) {
	var templates []*Template
	var errors []error

	for _, col := range columns {
		t, err := TemplateCompile('{', '}', col.Format)
		if err != nil {
			errors = append(errors, err)
			continue
		}

		templates = append(templates, t)
	}

	return templates, errors
}

func ConfigAligns(columns []ConfigColumn) []Align {
	var alings []Align

	for _, col := range columns {
		alings = append(alings, col.Align)
	}

	return alings
}
