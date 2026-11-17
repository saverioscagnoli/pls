package main

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"strings"
)

type SplitMap[V any] map[string]V

func (m *SplitMap[V]) UnmarshalJSONFrom(dec *jsontext.Decoder) error {
	var raw map[string]V
	if err := json.UnmarshalDecode(dec, &raw); err != nil {
		return err
	}

	*m = make(SplitMap[V], len(raw))
	for keys, val := range raw {
		for key := range strings.FieldsSeq(keys) {
			(*m)[key] = val
		}
	}
	return nil
}
