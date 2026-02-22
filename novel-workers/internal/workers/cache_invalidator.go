package workers

import (
	"context"
	"encoding/json"
	"fmt"
	"log"

	"github.com/nats-io/nats.go"
	"github.com/redis/go-redis/v9"

	"github.com/novel-platform/novel-workers/internal/models"
)

type CacheInvalidator struct {
	rdb *redis.Client
}

func NewCacheInvalidator(rdb *redis.Client) *CacheInvalidator {
	return &CacheInvalidator{rdb: rdb}
}

func (c *CacheInvalidator) Start(ctx context.Context, js nats.JetStreamContext) error {
	_, err := js.QueueSubscribe("novel.>", "cache-invalidator", func(msg *nats.Msg) {
		c.handleEvent(ctx, msg)
	}, nats.Durable("cache-invalidator"), nats.ManualAck())
	if err != nil {
		return fmt.Errorf("subscribe: %w", err)
	}

	log.Printf("Cache invalidator started")

	<-ctx.Done()
	return nil
}

func (c *CacheInvalidator) handleEvent(ctx context.Context, msg *nats.Msg) {
	var event models.EventPayload
	if err := json.Unmarshal(msg.Data, &event); err != nil {
		log.Printf("cache invalidator: unmarshal error: %v", err)
		msg.Nak()
		return
	}

	keys := c.keysForEvent(event)

	if len(keys) > 0 {
		pipe := c.rdb.Pipeline()
		for _, key := range keys {
			pipe.Del(ctx, key)
		}
		if _, err := pipe.Exec(ctx); err != nil {
			log.Printf("cache invalidator: redis pipeline error: %v", err)
			msg.Nak()
			return
		}
	}

	msg.Ack()
}

func (c *CacheInvalidator) keysForEvent(event models.EventPayload) []string {
	id := event.EntityID
	var keys []string

	switch event.EventType {
	case "novel.novel.created", "novel.novel.updated", "novel.novel.deleted":
		keys = append(keys,
			fmt.Sprintf("novel:novel:%s:detail", id),
			fmt.Sprintf("novel:novel:%s:chapters", id),
			fmt.Sprintf("novel:novel:%s:rating", id),
			"novel:novel:trending",
			"novel:novel:latest",
		)
	case "novel.chapter.created", "novel.chapter.updated", "novel.chapter.deleted":
		keys = append(keys, fmt.Sprintf("novel:chapter:%s:detail", id))
	case "novel.review.created", "novel.review.updated", "novel.review.deleted":
		keys = append(keys, fmt.Sprintf("novel:review:%s:detail", id))
	case "novel.forum.thread.created":
		keys = append(keys, fmt.Sprintf("novel:forum:thread:%s:replies", id))
	case "novel.forum.reply.created":
		keys = append(keys, fmt.Sprintf("novel:forum:thread:%s:replies", id))
	}

	return keys
}
