package config

import (
	"fmt"
	"os"
)

type Config struct {
	DatabaseURL    string
	RedisURL       string
	NatsURL        string
	MeilisearchURL string
	MeilisearchKey string
}

func Load() *Config {
	return &Config{
		DatabaseURL:    getEnv("DATABASE_URL", "postgres://novel:novel_password@localhost:5432/novel_platform"),
		RedisURL:       getEnv("REDIS_URL", "redis://localhost:6379"),
		NatsURL:        getEnv("NATS_URL", "nats://localhost:4222"),
		MeilisearchURL: getEnv("MEILISEARCH_URL", "http://localhost:7700"),
		MeilisearchKey: getEnv("MEILISEARCH_API_KEY", ""),
	}
}

func (c *Config) Validate() error {
	if c.DatabaseURL == "" {
		return fmt.Errorf("DATABASE_URL is required")
	}
	if c.NatsURL == "" {
		return fmt.Errorf("NATS_URL is required")
	}
	return nil
}

func getEnv(key, fallback string) string {
	if val := os.Getenv(key); val != "" {
		return val
	}
	return fallback
}
