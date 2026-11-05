package config

import "pls/ui"

type ConfigListColumn struct {
	Header *string  `json:"header"`
	Format string   `json:"format"`
	Align  ui.Align `json:"align"`
}

type ConfigListStyleItem struct {
	Icon  string `json:"icon"`
	Style string `json:"style"`
}

type ConfigListFileStyles struct {
	Default    ConfigListStyleItem           `json:"default"`
	Names      SplitMap[ConfigListStyleItem] `json:"names"`
	Extensions SplitMap[ConfigListStyleItem] `json:"extensions"`
	Kinds      SplitMap[ConfigListStyleItem] `json:"kinds"`
}

type ConfigListStyles struct {
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
	Columns     []ConfigListColumn   `json:"columns"`
	Gap         uint                 `json:"gap"`
	ModifiedFmt string               `json:"modified-fmt"`
	Files       ConfigListFileStyles `json:"files"`
	Styles      ConfigListStyles     `json:"styles"`
}

func configListDefault() ConfigList {
	return ConfigList{
		Columns: []ConfigListColumn{
			{Format: "{icon} {name}", Align: ui.AlignLeft},
			{Format: "{permissions}", Align: ui.AlignLeft},
			{Format: "{size}", Align: ui.AlignRight},
			{Format: "{modified}", Align: ui.AlignLeft},
			{Format: "{owner} ({group})", Align: ui.AlignLeft},
			{Format: "-> {nlink}", Align: ui.AlignLeft},
		},
		Gap:         2,
		ModifiedFmt: "Jan 02 15:04",
		Files: ConfigListFileStyles{
			Default: ConfigListStyleItem{Icon: "f"},
			Kinds: SplitMap[ConfigListStyleItem]{
				"file":      ConfigListStyleItem{Icon: "f"},
				"directory": ConfigListStyleItem{Icon: "d"},
			},
		},
		Styles: ConfigListStyles{},
	}
}
