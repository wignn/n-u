package main

import (
	"context"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/meilisearch/meilisearch-go"
	"github.com/redis/go-redis/v9"

	"github.com/novel-platform/novel-workers/internal/config"
	"github.com/novel-platform/novel-workers/internal/db"
	natsutil "github.com/novel-platform/novel-workers/internal/nats"
	"github.com/novel-platform/novel-workers/internal/workers"
)

func main() {
	cfg := config.Load()
	if err := cfg.Validate(); err != nil {
		log.Fatalf("config error: %v", err)
	}

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	pool, err := db.NewPool(ctx, cfg.DatabaseURL)
	if err != nil {
		log.Fatalf("database error: %v", err)
	}
	defer pool.Close()
	log.Printf("PostgreSQL connected")

	nc, js, err := natsutil.Connect(cfg.NatsURL)
	if err != nil {
		log.Fatalf("nats error: %v", err)
	}
	defer nc.Close()
	log.Printf("NATS connected")

	err = natsutil.EnsureStream(js, "NOVEL", []string{"novel.>"})
	if err != nil {
		log.Fatalf("stream setup error: %v", err)
	}

	opts, err := redis.ParseURL(cfg.RedisURL)
	if err != nil {
		log.Fatalf("redis url error: %v", err)
	}
	rdb := redis.NewClient(opts)
	defer rdb.Close()
	log.Printf("Redis connected")

	searchClient := meilisearch.New(cfg.MeilisearchURL, meilisearch.WithAPIKey(cfg.MeilisearchKey))
	log.Printf("Meilisearch connected")

	go func() {
		indexer := workers.NewSearchIndexer(pool, searchClient)
		if err := indexer.Start(ctx, js); err != nil {
			log.Printf("search indexer error: %v", err)
		}
	}()

	go func() {
		notifier := workers.NewNotificationWorker(pool)
		if err := notifier.Start(ctx, js); err != nil {
			log.Printf("notification worker error: %v", err)
		}
	}()

	go func() {
		invalidator := workers.NewCacheInvalidator(rdb)
		if err := invalidator.Start(ctx, js); err != nil {
			log.Printf("cache invalidator error: %v", err)
		}
	}()

	go func() {
		mod := workers.NewModerationWorker(pool)
		if err := mod.Start(ctx, js); err != nil {
			log.Printf("moderation worker error: %v", err)
		}
	}()

	log.Printf("All workers started")

	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)
	<-sigCh

	log.Printf("Shutting down workers...")
	cancel()
}
