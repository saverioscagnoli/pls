package config

import (
	"encoding/json"
	"strings"
)

type SplitMap[V any] map[string]V

func (m *SplitMap[V]) UnmarshalJSON(data []byte) error {
	var raw map[string]V

	if err := json.Unmarshal(data, &raw); err != nil {
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
