package config

import "pls/ui"

type ConfigDisksColumn struct {
	Header *string  `json:"header"`
	Format string   `json:"format"`
	Align  ui.Align `json:"align"`
}

type ConfigDisksBarThreshold struct {
	Min       float64  `json:"min"`
	Color     ui.Color `json:"color"`
	TextColor ui.Color `json:"text-color"`
}

type ConfigDisksBar struct {
	Width      uint                      `json:"width"`
	Thresholds []ConfigDisksBarThreshold `json:"thresholds"`
}

type ConfigDisks struct {
	Columns  []ConfigDisksColumn `json:"columns"`
	Gap      uint                `json:"gap"`
	SizeUnit ui.SizeFormat       `json:"size-unit"`
	Bar      ConfigDisksBar      `json:"bar"`
}

func configDisksDefault() ConfigDisks {
	return ConfigDisks{
		Columns: []ConfigDisksColumn{
			{Header: ui.StrPtr("Device"), Format: "{dev}"},
			{Header: ui.StrPtr("Mountpoint"), Format: "{mount}"},
			{Header: ui.StrPtr("Type"), Format: "{fstype}"},
			{Header: ui.StrPtr("Usage"), Format: "{bar}"},
			{Format: "{used} / {total}", Align: ui.AlignRight},
		},
		Gap:      2,
		SizeUnit: ui.SizeFormat{Unit: ui.UnitAuto, Decimals: 1},
		Bar: ConfigDisksBar{
			Width: 20,
			Thresholds: []ConfigDisksBarThreshold{
				{Min: 0.00, Color: ui.ColorGreen, TextColor: ui.ColorWhite},
				{Min: 0.75, Color: ui.ColorYellow, TextColor: ui.ColorBlack},
				{Min: 0.90, Color: ui.ColorRed, TextColor: ui.ColorWhite},
			},
		},
	}
}
