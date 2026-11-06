package ui

import (
	"fmt"
	"strings"
)

var styleByName = map[string]Style{
	"bold":          StyleBold,
	"dim":           StyleDim,
	"italic":        StyleItalic,
	"underline":     StyleUnderline,
	"blink":         StyleBlink,
	"reverse":       StyleReverse,
	"hidden":        StyleHidden,
	"strikethrough": StyleStrikethrough,
}

type Spec struct {
	prefix string
}

func ParseSpec(spec string, palette map[string]Color) (Spec, error) {
	var b strings.Builder
	on := false

	for word := range strings.FieldsSeq(spec) {
		if word == "on" {
			if on {
				return Spec{}, fmt.Errorf("unexpected \"on\" in %q", spec)
			}
			on = true
			continue
		}

		if st, ok := styleByName[normalizeName(word)]; ok && !on {
			b.WriteString(code(int(st)))
			continue
		}

		col, ok := palette[word]
		if !ok {
			var err error
			if col, err = ParseColor(word); err != nil {
				return Spec{}, err
			}
		}

		if on {
			b.WriteString(col.bg())
			on = false
		} else {
			b.WriteString(col.fg())
		}
	}

	if on {
		return Spec{}, fmt.Errorf("\"on\" without a color in %q", spec)
	}

	return Spec{prefix: b.String()}, nil
}

func (s Spec) Apply(text string) string {
	if s.prefix == "" {
		return text
	}

	return s.prefix + text + Reset
}

func (s Spec) Merge(o Spec) Spec {
	return Spec{prefix: s.prefix + o.prefix}
}
