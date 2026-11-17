package main

type ConfigListStyleItem struct {
	Icon  string `json:"icon"`
	Style string `json:"style"`
}

type ConfigListStyles struct {
	Default    ConfigListStyleItem
	Names      SplitMap[ConfigListStyleItem]
	Extensions SplitMap[ConfigListStyleItem]
	Kind       SplitMap[ConfigListStyleItem]
}

type ConfigListVariableStyles struct {
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
	Columns     []ConfigColumn `json:"columns"`
	Gap         uint           `json:"gap"`
	ModifiedFmt string         `json:"modified-fmt"`
	SizeFmt     SizeFormat     `json:"size-fmt"`
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
