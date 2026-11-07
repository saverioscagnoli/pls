package config

import "pls/ui"

type ConfigDisksColumn struct {
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
	Columns []ConfigDisksColumn `json:"columns"`
	Gap     uint                `json:"gap"`
	Bar     ConfigDisksBar      `json:"bar"`
}

func configDisksDefault() ConfigDisks {
	return ConfigDisks{
		Columns: []ConfigDisksColumn{
			{Format: "{dev}"},
			{Format: "{mount}"},
			{Format: "{fstype}"},
			{Format: "{bar}"},
			{Format: "{used} / {total}"},
		},
		Gap: 2,
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
