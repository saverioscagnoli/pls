package main

import (
	"encoding/json/jsontext"
	"encoding/json/v2"

	"github.com/fatih/color"
)

type StyleRule struct {
	Default *color.Color          `json:"default"`
	Kinds   SplitMap[color.Color] `json:"kinds"`
}

func (s *StyleRule) UnmarshalJSONFrom(dec *jsontext.Decoder) error {
	if dec.PeekKind() == '"' {
		return json.UnmarshalDecode(dec, &s.Default)
	}

	type plain StyleRule
	return json.UnmarshalDecode(dec, (*plain)(s))
}

func ApplyColor(c *color.Color, text string) string {
	if c == nil {
		return text
	}

	return c.Sprint(text)
}

func (s *StyleRule) Apply(kind DirEntryKind, text string) string {
	if c, ok := s.Kinds[string(kind)]; ok {
		return c.Sprint(text)
	}
	return ApplyColor(s.Default, text)
}
