package template

import (
	"fmt"
	"strconv"
	"strings"
)

type part struct {
	text   string
	isVar  bool
	repeat string
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

			raw := buf.String()
			p := part{text: strings.TrimSpace(raw), isVar: true}

			if j := strings.LastIndexByte(raw, ':'); j >= 0 {
				p.repeat = raw[:j]
				p.text = strings.TrimSpace(raw[j+1:])

				if p.repeat == "" {
					return nil, fmt.Errorf("empty repeat string at position %d", i)
				}
			}

			if p.text == "" {
				return nil, fmt.Errorf("empty repeat string at position %d", i)
			}

			parts = append(parts, p)
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

		if p.repeat != "" {
			n, err := strconv.Atoi(v)
			if err != nil || n < 0 {
				return "", fmt.Errorf("variable %q is not a valid count: %q", p.text, v)
			}

			sb.WriteString(strings.Repeat(p.repeat, n))
			continue
		}

		sb.WriteString(v)
	}

	return sb.String(), nil
}
