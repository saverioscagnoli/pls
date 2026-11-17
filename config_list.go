package main

import (
	"path/filepath"

	"github.com/fatih/color"
)

type ConfigListStyleItem struct {
	Icon  string       `json:"icon"`
	Style *color.Color `json:"style"`
}

type ConfigListStyles struct {
	Default    ConfigListStyleItem
	Names      SplitMap[ConfigListStyleItem]
	Extensions SplitMap[ConfigListStyleItem]
	Kind       SplitMap[ConfigListStyleItem]
}

func (s *ConfigListStyles) Resolve(name string, kind DirEntryKind) ConfigListStyleItem {
	if item, ok := s.Names[name]; ok {
		return item
	}

	if item, ok := s.Extensions[filepath.Ext(name)]; ok {
		return item
	}

	if item, ok := s.Kind[string(kind)]; ok {
		return item
	}

	return s.Default
}

func (s *ConfigListStyles) ColorFor(name string, kind DirEntryKind, variable *StyleRule) *color.Color {
	if item, ok := s.Names[name]; ok && item.Style != nil {
		return item.Style
	}

	if item, ok := s.Extensions[filepath.Ext(name)]; ok && item.Style != nil {
		return item.Style
	}

	if variable != nil {
		if c, ok := variable.Kinds[string(kind)]; ok {
			return &c
		}
	}

	if item, ok := s.Kind[string(kind)]; ok && item.Style != nil {
		return item.Style
	}

	if variable != nil && variable.Default != nil {
		return variable.Default
	}

	return s.Default.Style
}

type ConfigListVariableStyles struct {
	Depth       StyleRule `json:"depth"`
	Name        StyleRule `json:"name"`
	Kind        StyleRule `json:"kind"`
	Icon        StyleRule `json:"icon"`
	Permissions StyleRule `json:"permissions"`
	Size        StyleRule `json:"size"`
	Modified    StyleRule `json:"modified"`
	NLink       StyleRule `json:"nlink"`
	Owner       StyleRule `json:"owner"`
	Group       StyleRule `json:"group"`
}

type ConfigList struct {
	Columns     []ConfigColumn           `json:"columns"`
	Gap         uint                     `json:"gap"`
	ModifiedFmt string                   `json:"modified-fmt"`
	SizeFmt     SizeFormat               `json:"size-fmt"`
	Styles      ConfigListStyles         `json:"styles"`
	VarStyles   ConfigListVariableStyles `json:"var-styles"`
}

func ConfigListDefault() ConfigList {
	return ConfigList{
		Columns: []ConfigColumn{
			{Format: "{  :depth}{name}"},
			{Format: "{permissions}"},
			{Format: "{size}", Align: AlignRight},
			{Format: "{modified}"},
			{Format: "{owner} ({group})"},
			{Format: "-> {nlink}"},
		},
		Gap:         2,
		ModifiedFmt: "Jan 02 15:04",
		SizeFmt:     SizeFormat{Unit: UnitAuto, Decimals: 1},
	}
}
