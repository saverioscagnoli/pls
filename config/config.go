package config

import (
	"encoding/json"
	"errors"
	"os"
	"path/filepath"
	"pls/ui"
)

type ListColumn struct {
	Header *string  `json:"header"`
	Format string   `json:"format"`
	Align  ui.Align `json:"align"`
}

type ListIcons struct {
	Default    string            `json:"default"`
	Names      map[string]string `json:"names"`
	Extensions map[string]string `json:"extensions"`
	Kinds      map[string]string `json:"kinds"`
}

type List struct {
	Columns     []ListColumn `json:"columns"`
	Gap         uint         `json:"gap"`
	ModifiedFmt string       `json:"modified_fmt"`
	Icons       ListIcons    `json:"icons"`
}

type Find struct {
}

type Config struct {
	Palette map[string]ui.Color `json:"palette"`
	List    List                `json:"list"`
	Find    Find                `json:"find"`
}

func Default() *Config {
	return &Config{
		List: List{
			Columns: []ListColumn{
				{Header: nil, Align: ui.AlignLeft, Format: "{icon} {name}"},
				{Header: nil, Align: ui.AlignLeft, Format: "{permissions}"},
				{Header: nil, Align: ui.AlignRight, Format: "{size}"},
				{Header: nil, Align: ui.AlignLeft, Format: "{modified}"},
				{Header: nil, Align: ui.AlignLeft, Format: "{nlink}"},
			},
			Gap:         2,
			ModifiedFmt: "Jan _ 15:04",
			Icons: ListIcons{
				Default:    "f",
				Names:      map[string]string{},
				Extensions: map[string]string{},
				Kinds: map[string]string{
					"file":      "f",
					"directory": "d",
				},
			},
		},
	}
}

func Save(path string, cfg *Config) error {
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return err
	}

	data, err := json.MarshalIndent(cfg, "", " ")

	if err != nil {
		return err
	}

	return os.WriteFile(path, data, 0o644)
}

func Parse(path string) (*Config, error) {
	content, err := os.ReadFile(path)

	if errors.Is(err, os.ErrNotExist) {
		cfg := Default()

		if err := Save(path, cfg); err != nil {
			return nil, err
		}

		return cfg, nil
	}

	if err != nil {
		return nil, err
	}

	cfg := Default()

	if err := json.Unmarshal(content, cfg); err != nil {
		return nil, err
	}

	return cfg, nil
}
