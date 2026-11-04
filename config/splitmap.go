package config

import (
	"encoding/json"
	"strings"
)

type SplitMap map[string]string

func (m *SplitMap) UnmarshalJSON(data []byte) error {
	var raw map[string]string

	if err := json.Unmarshal(data, &raw); err != nil {
		return err
	}

	*m = make(SplitMap)

	for keys, val := range raw {
		for ext := range strings.FieldsSeq(keys) {
			(*m)[ext] = val
		}
	}

	return nil
}
