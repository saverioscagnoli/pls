package main

import (
	"fmt"
	"strconv"
	"strings"
)

type Unit int

const (
	UnitAuto Unit = iota
	UnitBytes
	UnitKilobytes
	UnitMegabytes
	UnitGigabytes
	UnitTerabytes
)

var unitByName = map[string]Unit{
	"auto":  UnitAuto,
	"bytes": UnitBytes,
	"kb":    UnitKilobytes,
	"mb":    UnitMegabytes,
	"gb":    UnitGigabytes,
	"tb":    UnitTerabytes,
}

var unitNames = map[Unit]string{
	UnitAuto:      "auto",
	UnitBytes:     "bytes",
	UnitKilobytes: "kb",
	UnitMegabytes: "mb",
	UnitGigabytes: "gb",
	UnitTerabytes: "tb",
}

var unitSymbols = map[Unit]string{
	UnitBytes:     "B",
	UnitKilobytes: "KB",
	UnitMegabytes: "MB",
	UnitGigabytes: "GB",
	UnitTerabytes: "TB",
}

func (u *Unit) UnmarshalText(b []byte) error {
	v, ok := unitByName[strings.ToLower(string(b))]
	if !ok {
		return fmt.Errorf("invalid size unit: %q", b)
	}

	*u = v
	return nil
}

func (u Unit) MarshalText() ([]byte, error) {
	s, ok := unitNames[u]
	if !ok {
		return nil, fmt.Errorf("invalid size unit: %d", u)
	}

	return []byte(s), nil
}

func (u Unit) String() string {
	if s, ok := unitNames[u]; ok {
		return s
	}

	return fmt.Sprintf("Unit(%d)", u)
}

type SizeFormat struct {
	Unit     Unit
	Decimals int
}

func (f *SizeFormat) UnmarshalText(b []byte) error {
	name, dec, hasDec := strings.Cut(strings.ToLower(string(b)), ".")

	u, ok := unitByName[name]
	if !ok {
		return fmt.Errorf("invalid size unit: %q", name)
	}

	d := 0
	if hasDec {
		n, err := strconv.Atoi(dec)
		if err != nil || n < 0 || n > 9 {
			return fmt.Errorf("invalid decimal count in %q (want 0-9)", b)
		}
		d = n
	}

	f.Unit, f.Decimals = u, d
	return nil
}

func (f SizeFormat) MarshalText() ([]byte, error) {
	name, ok := unitNames[f.Unit]
	if !ok {
		return nil, fmt.Errorf("invalid size unit: %d", f.Unit)
	}
	if f.Decimals == 0 {
		return []byte(name), nil
	}
	return []byte(name + "." + strconv.Itoa(f.Decimals)), nil
}

type Size struct {
	Bytes int64
}

func NewSize(bytes int64) Size {
	return Size{Bytes: bytes}
}

func divisor(u Unit) int64 {
	return int64(1) << (10 * (u - UnitBytes))
}

func (s Size) autoUnit() Unit {
	u := UnitBytes
	for u < UnitTerabytes && s.Bytes >= divisor(u+1) {
		u++
	}
	return u
}

func (s Size) Format(f SizeFormat) string {
	u := f.Unit
	if u == UnitAuto {
		u = s.autoUnit()
	}
	if u == UnitBytes {
		return fmt.Sprintf("%d B", s.Bytes)
	}
	return fmt.Sprintf("%.*f %s", f.Decimals, float64(s.Bytes)/float64(divisor(u)), unitSymbols[u])
}

func (s Size) String() string {
	return s.Format(SizeFormat{Unit: UnitAuto, Decimals: 1})
}
