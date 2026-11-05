package config

import (
	"bytes"
	"encoding/json"
)

type StyleRule struct {
	Default string           `json:"default"`
	Kinds   SplitMap[string] `json:"kinds,omitempty"`
}

func (r StyleRule) For(kind string) string {
	if s, ok := r.Kinds[kind]; ok {
		return s
	}

	return r.Default
}

func (r *StyleRule) UnmarshalJSON(data []byte) error {
	trimmed := bytes.TrimSpace(data)

	if len(trimmed) > 0 && trimmed[0] == '"' {
		r.Kinds = nil
		return json.Unmarshal(trimmed, &r.Default)
	}

	type plain StyleRule
	var p plain

	dec := json.NewDecoder(bytes.NewReader(trimmed))
	dec.DisallowUnknownFields()

	if err := dec.Decode(&p); err != nil {
		return err
	}

	*r = StyleRule(p)
	return nil
}

func (r StyleRule) MarshalJSON() ([]byte, error) {
	if len(r.Kinds) == 0 {
		return json.Marshal(r.Default)
	}

	type plain StyleRule
	return json.Marshal(plain(r))
}
