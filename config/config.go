package config

import (
	"bytes"
	"encoding/json"
	"errors"
	"os"
	"path/filepath"
	"pls/ui"
)

type ConfigFind struct {
}

type Config struct {
	Palette map[string]ui.Color `json:"palette"`
	List    ConfigList          `json:"list"`
	Find    ConfigFind          `json:"find"`
	Disks   ConfigDisks         `json:"disks"`
}

func Default() *Config {
	return &Config{
		List:  configListDefault(),
		Disks: configDisksDefault(),
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

	dec := json.NewDecoder(bytes.NewReader(content))
	dec.DisallowUnknownFields()

	if err := dec.Decode(cfg); err != nil {
		return nil, err
	}

	return cfg, nil
}
