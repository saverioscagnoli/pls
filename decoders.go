package main

import (
	"encoding/json/v2"
	"fmt"
	"strconv"
	"strings"

	"github.com/fatih/color"
)

var NamedColors = map[string][]color.Attribute{
	"black":   {color.FgBlack},
	"red":     {color.FgRed},
	"green":   {color.FgGreen},
	"yellow":  {color.FgYellow},
	"blue":    {color.FgBlue},
	"magenta": {color.FgMagenta},
	"cyan":    {color.FgCyan},
	"white":   {color.FgWhite},

	"bright-black":   {color.FgHiBlack},
	"bright-red":     {color.FgHiRed},
	"bright-green":   {color.FgHiGreen},
	"bright-yellow":  {color.FgHiYellow},
	"bright-blue":    {color.FgHiBlue},
	"bright-magenta": {color.FgHiMagenta},
	"bright-cyan":    {color.FgHiCyan},
	"bright-white":   {color.FgHiWhite},

	"bg-black":   {color.BgBlack},
	"bg-red":     {color.BgRed},
	"bg-green":   {color.BgGreen},
	"bg-yellow":  {color.BgYellow},
	"bg-blue":    {color.BgBlue},
	"bg-magenta": {color.BgMagenta},
	"bg-cyan":    {color.BgCyan},
	"bg-white":   {color.BgWhite},

	"bg-bright-black":   {color.BgHiBlack},
	"bg-bright-red":     {color.BgHiRed},
	"bg-bright-green":   {color.BgHiGreen},
	"bg-bright-yellow":  {color.BgHiYellow},
	"bg-bright-blue":    {color.BgHiBlue},
	"bg-bright-magenta": {color.BgHiMagenta},
	"bg-bright-cyan":    {color.BgHiCyan},
	"bg-bright-white":   {color.BgHiWhite},
}

func DecodeColor(data []byte, dst *color.Color) error {
	var spec string
	if err := json.Unmarshal(data, &spec); err != nil {
		return err
	}

	tokens := strings.Fields(strings.ToLower(spec))

	if len(tokens) == 0 {
		return fmt.Errorf("color style must contain at least one token")
	}

	style := color.New()

	for len(tokens) > 0 {
		token := tokens[0]
		tokens = tokens[1:]
		background := token == "on"

		if background {
			if len(tokens) == 0 {
				return fmt.Errorf("expected a background color after 'on'")
			}

			token = tokens[0]
			tokens = tokens[1:]
		}

		if strings.HasPrefix(token, "#") {
			if len(token) != 7 {
				return fmt.Errorf("invalid hex color %q, expected #RRGGBB", token)
			}

			rgb, err := strconv.ParseUint(token[1:], 16, 24)
			if err != nil {
				return fmt.Errorf("invalid hex color %q: %w", token, err)
			}

			r, g, b := int(rgb>>16), int((rgb>>8)&0xff), int(rgb&0xff)

			if background {
				style.AddBgRGB(r, g, b)
			} else {
				style.AddRGB(r, g, b)
			}

			continue
		}

		if strings.HasPrefix(token, "bg-") {
			return fmt.Errorf("use 'on %s' for a background color", strings.TrimPrefix(token, "bg-"))
		}

		name := token

		if background {
			name = "bg-" + token
		}

		attrs, ok := NamedColors[name]
		if !ok {
			if background {
				return fmt.Errorf("unknown background color %q", token)
			}
			return fmt.Errorf("unknown color or style %q", token)
		}

		style.Add(attrs...)
	}

	*dst = *style
	return nil
}
