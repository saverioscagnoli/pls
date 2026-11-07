package config

import "pls/ui"

type BarThreshold struct {
	Min       float64  `json:"min"`
	Color     ui.Color `json:"color"`
	TextColor ui.Color `json:"text-color"`
}

type ConfigDisks struct {
	BarWidth   uint           `json:"bar-width"`
	Thresholds []BarThreshold `json:"thresholds"`
}

func configDisksDefault() ConfigDisks {
	return ConfigDisks{
		BarWidth: 20,
		Thresholds: []BarThreshold{
			{Min: 0.00, Color: ui.ColorGreen, TextColor: ui.ColorWhite},
			{Min: 0.75, Color: ui.ColorYellow, TextColor: ui.ColorBlack},
			{Min: 0.90, Color: ui.ColorRed, TextColor: ui.ColorWhite},
		},
	}
}
