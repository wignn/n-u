package models

import (
	"time"
)

type EventPayload struct {
	EntityID  string    `json:"entity_id"`
	EventType string    `json:"event_type"`
	Timestamp time.Time `json:"timestamp"`
}
