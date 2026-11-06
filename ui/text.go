package ui

import (
	"encoding/json"
	"fmt"
	"strconv"
	"strings"
)

type Color int

const Reset = "\033[0m"

const (
	ColorBlack   Color = 30
	ColorRed     Color = 31
	ColorGreen   Color = 32
	ColorYellow  Color = 33
	ColorBlue    Color = 34
	ColorMagenta Color = 35
	ColorCyan    Color = 36
	ColorWhite   Color = 37

	ColorGray          Color = 90
	ColorBrightBlack   Color = ColorGray
	ColorBrightRed     Color = 91
	ColorBrightGreen   Color = 92
	ColorBrightYellow  Color = 93
	ColorBrightBlue    Color = 94
	ColorBrightMagenta Color = 95
	ColorBrightCyan    Color = 96
	ColorBrightWhite   Color = 97
)

// Bit 24 marks truecolor value
const rgbFlag = 1 << 24

func RGB(r uint8, g uint8, b uint8) Color {
	return rgbFlag | Color(r)<<16 | Color(g)<<8 | Color(b)
}

func (c Color) IsRGB() bool {
	return c&rgbFlag != 0
}

func (c Color) RGB() (r uint8, g uint8, b uint8) {
	return uint8(c >> 16), uint8(c >> 8), uint8(c)
}

var colorNames = map[Color]string{
	ColorBlack:         "black",
	ColorRed:           "red",
	ColorGreen:         "green",
	ColorYellow:        "yellow",
	ColorBlue:          "blue",
	ColorMagenta:       "magenta",
	ColorCyan:          "cyan",
	ColorWhite:         "white",
	ColorGray:          "gray",
	ColorBrightRed:     "bright-red",
	ColorBrightGreen:   "bright-green",
	ColorBrightYellow:  "bright-yellow",
	ColorBrightBlue:    "bright-blue",
	ColorBrightMagenta: "bright-magenta",
	ColorBrightCyan:    "bright-cyan",
	ColorBrightWhite:   "bright-white",
}

func normalizeName(s string) string {
	s = strings.ToLower(strings.TrimSpace(s))
	return strings.NewReplacer("_", "", "-", "", " ", "").Replace(s)
}

var colorByName = func() map[string]Color {
	m := map[string]Color{
		"grey":        ColorGray,
		"brightblack": ColorBrightBlack,
	}

	for c, name := range colorNames {
		m[normalizeName(name)] = c
	}

	return m
}()

func ParseColor(s string) (Color, error) {
	s = strings.TrimSpace(s)

	if hex, ok := strings.CutPrefix(s, "#"); ok {
		if len(hex) == 3 {
			hex = string([]byte{hex[0], hex[0], hex[1], hex[1], hex[2], hex[2]})
		}

		if len(hex) != 6 {
			return 0, fmt.Errorf("invalid hex color %q", s)
		}

		v, err := strconv.ParseUint(hex, 16, 32)

		if err != nil {
			return 0, fmt.Errorf("invalid hex color %q", s)
		}

		return RGB(uint8(v>>16), uint8(v>>8), uint8(v)), nil
	}

	if c, ok := colorByName[normalizeName(s)]; ok {
		return c, nil
	}

	return 0, fmt.Errorf("unknown color %q", s)
}

func (c *Color) UnmarshalJSON(data []byte) error {
	if string(data) == "null" {
		return nil
	}

	var s string

	if err := json.Unmarshal(data, &s); err == nil {
		parsed, err := ParseColor(s)

		if err != nil {
			return err
		}

		*c = parsed
		return nil
	}

	var rgb []int
	if err := json.Unmarshal(data, &rgb); err == nil {
		if len(rgb) != 3 {
			return fmt.Errorf("color array must have 3 elements, got %d", len(rgb))
		}

		for _, v := range rgb {
			if v < 0 || v > 255 {
				return fmt.Errorf("color component %d out of range 0-255", v)
			}
		}

		*c = RGB(uint8(rgb[0]), uint8(rgb[1]), uint8(rgb[2]))
		return nil
	}

	return fmt.Errorf(`invalid color %s: want a name, "#rrggbb" or [r, g, b] (0-255)`, data)
}

func (c Color) MarshalJSON() ([]byte, error) {
	if c.IsRGB() {
		r, g, b := c.RGB()
		return json.Marshal(fmt.Sprintf("#%02x%02x%02x", r, g, b))
	}

	if name, ok := colorNames[c]; ok {
		return json.Marshal(name)
	}

	return nil, fmt.Errorf("cannot marshal color %d", int(c))
}

func code(n int) string {
	return "\033[" + strconv.Itoa(n) + "m"
}

func (c Color) fg() string {
	if c.IsRGB() {
		r, g, b := c.RGB()
		return fmt.Sprintf("\033[38;2;%d;%d;%dm", r, g, b)
	}
	return code(int(c))
}

func (c Color) bg() string {
	if c.IsRGB() {
		r, g, b := c.RGB()
		return fmt.Sprintf("\033[48;2;%d;%d;%dm", r, g, b)
	}

	// background color is just foreground +10
	return code(int(c) + 10)
}

func (c Color) FG() string { return c.fg() }
func (c Color) BG() string { return c.bg() }

func Foreground(s string, c Color) string {
	return c.fg() + s + Reset
}

func Background(s string, c Color) string {
	return c.bg() + s + Reset
}

func Colors(s string, fg Color, bg Color) string {
	return fg.fg() + bg.bg() + s + Reset
}

type Style int

const (
	StyleBold          Style = 1
	StyleDim           Style = 2
	StyleItalic        Style = 3
	StyleUnderline     Style = 4
	StyleBlink         Style = 5
	StyleReverse       Style = 7
	StyleHidden        Style = 8
	StyleStrikethrough Style = 9
)

func Stylize(s string, styles ...Style) string {
	var prefix strings.Builder

	for _, st := range styles {
		prefix.WriteString(code(int(st)))
	}

	return prefix.String() + s + Reset
}
