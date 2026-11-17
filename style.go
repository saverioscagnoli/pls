package main

import "github.com/fatih/color"

type StyleRule struct {
	Default color.Color `json:"default"`
	Kinds   SplitMap[color.Color]
}
