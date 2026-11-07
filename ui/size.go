package ui

import "fmt"

type Unit int

const (
	UnitAuto      Unit = 1
	UnitBytes     Unit = 2
	UnitKilobytes Unit = 3
	UnitMegabytes Unit = 4
	UnitGigabytes Unit = 5
	UnitTerabytes Unit = 6
)

var unitNames = map[Unit]string{
	UnitBytes:     "B",
	UnitKilobytes: "KB",
	UnitMegabytes: "MB",
	UnitGigabytes: "GB",
	UnitTerabytes: "TB",
}

type Size struct {
	Bytes uint64
}

func NewSize(bytes uint64) Size {
	return Size{Bytes: bytes}
}

func divisor(u Unit) uint64 {
	d := uint64(1)

	for i := UnitBytes; i < u; i++ {
		d *= 1024
	}

	return d
}

func (s Size) autoUnit() Unit {
	u := UnitBytes

	for u < UnitTerabytes && s.Bytes >= divisor(u+1) {
		u++
	}

	return u
}

func (s Size) Format(u Unit) string {
	if u == UnitAuto {
		u = s.autoUnit()
	}

	if u == UnitBytes {
		return fmt.Sprintf("%d %s", s.Bytes, "B")
	}

	return fmt.Sprintf("%1.f %s", float64(s.Bytes)/float64(divisor(u)), unitNames[u])
}

func (s Size) String() string {
	return s.Format(UnitAuto)
}
