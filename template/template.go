package template

import (
	"fmt"
	"strings"
)

type part struct {
	text  string
	isVar bool
}

type Template struct {
	parts []part
}

func Compile(o rune, c rune, text string) (*Template, error) {
	var parts []part
	var buf strings.Builder
	var inside bool

	for i, ch := range text {
		switch ch {
		case o:
			if inside {
				return nil, fmt.Errorf("unexpected %c at position %d", o, i)
			}

			if buf.Len() > 0 {
				parts = append(parts, part{text: buf.String()})
				buf.Reset()
			}

			inside = true

		case c:
			if !inside {
				return nil, fmt.Errorf("unexpected %c at position %d", c, i)
			}

			name := strings.TrimSpace(buf.String())

			if name == "" {
				return nil, fmt.Errorf("empty variable name at position %d", i)
			}

			parts = append(parts, part{text: buf.String(), isVar: true})
			buf.Reset()
			inside = false

		default:
			buf.WriteRune(ch)
		}
	}

	if inside {
		return nil, fmt.Errorf("missing closing delimiter %c", c)
	}

	if buf.Len() > 0 {
		parts = append(parts, part{text: buf.String()})
	}

	return &Template{parts: parts}, nil
}

func (t *Template) Format(vars map[string]string) (string, error) {
	var sb strings.Builder

	for _, p := range t.parts {
		if !p.isVar {
			sb.WriteString(p.text)
			continue
		}

		v, ok := vars[p.text]

		if !ok {
			return "", fmt.Errorf("missing variable %q", p.text)
		}

		sb.WriteString(v)
	}

	return sb.String(), nil
}
