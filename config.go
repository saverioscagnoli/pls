package main

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/fatih/color"
)

type ConfigColumn struct {
	Header *string `json:"header,omitzero"`
	Format string  `json:"format"`
	Align  Align   `json:"align,omitempty"`
}

type ConfigFind struct{}

type Config struct {
	Palette map[string]string `json:"palette"`
	List    ConfigList        `json:"list"`
	Find    ConfigFind        `json:"find"`
}

func ConfigDefault() Config {
	return Config{
		Palette: map[string]string{},
		List:    ConfigListDefault(),
		Find:    ConfigFind{},
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

	var paletteConfig struct {
		Palette map[string]string `json:"palette"`
	}
	if err := json.Unmarshal(bytes, &paletteConfig); err != nil {
		return nil, err
	}
	palette := make(map[string]string, len(paletteConfig.Palette))
	for name, value := range paletteConfig.Palette {
		nameKey := strings.ToLower(name)
		value = strings.TrimSpace(value)
		if len(strings.Fields(name)) != 1 || nameKey == "on" || strings.HasPrefix(nameKey, "#") || strings.HasPrefix(nameKey, "bg-") {
			return nil, fmt.Errorf("palette %q: invalid color name", name)
		}
		if _, exists := palette[nameKey]; exists {
			return nil, fmt.Errorf("palette %q: duplicate color name ignoring case", name)
		}
		if len(strings.Fields(value)) != 1 || strings.HasPrefix(strings.ToLower(value), "bg-") {
			return nil, fmt.Errorf("palette %q: expected a single foreground color", name)
		}
		data, err := json.Marshal(value)
		if err != nil {
			return nil, err
		}
		var c color.Color
		if err := DecodeColor(data, &c, nil); err != nil {
			return nil, fmt.Errorf("palette %q: %w", name, err)
		}
		palette[nameKey] = value
	}
	if err := json.Unmarshal(bytes, &config,
		json.RejectUnknownMembers(true),
		json.WithUnmarshalers(json.UnmarshalFunc(func(data []byte, dst *color.Color) error {
			return DecodeColor(data, dst, palette)
		})),
	); err != nil {
		return nil, err
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
