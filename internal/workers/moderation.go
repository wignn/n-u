package workers

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/nats-io/nats.go"

	"github.com/novel-platform/novel-workers/internal/models"
)

type ModerationWorker struct {
	db *pgxpool.Pool
}

func NewModerationWorker(db *pgxpool.Pool) *ModerationWorker {
	return &ModerationWorker{db: db}
}

func (m *ModerationWorker) Start(ctx context.Context, js nats.JetStreamContext) error {
	subjects := []string{
		"novel.comment.created",
		"novel.review.created",
		"novel.forum.thread.created",
		"novel.forum.reply.created",
		"novel.report.created",
	}

	for _, subj := range subjects {
		s := subj
		_, err := js.QueueSubscribe(s, "moderation-worker", func(msg *nats.Msg) {
			m.handleEvent(ctx, msg)
		}, nats.Durable("mod-"+s), nats.ManualAck())
		if err != nil {
			return fmt.Errorf("subscribe %s: %w", s, err)
		}
	}

	log.Printf("Moderation worker started")

	<-ctx.Done()
	return nil
}

func (m *ModerationWorker) handleEvent(ctx context.Context, msg *nats.Msg) {
	var event models.EventPayload
	if err := json.Unmarshal(msg.Data, &event); err != nil {
		log.Printf("moderation worker: unmarshal error: %v", err)
		msg.Nak()
		return
	}

	switch event.EventType {
	case "novel.comment.created", "novel.review.created",
		"novel.forum.thread.created", "novel.forum.reply.created":
		if err := m.checkRateAbuse(ctx, event); err != nil {
			log.Printf("moderation worker: rate abuse check error: %v", err)
		}
	case "novel.report.created":
		if err := m.checkReportThreshold(ctx, event.EntityID); err != nil {
			log.Printf("moderation worker: report threshold error: %v", err)
		}
	}

	msg.Ack()
}

func (m *ModerationWorker) checkRateAbuse(ctx context.Context, event models.EventPayload) error {
	var userID string
	var table string

	switch event.EventType {
	case "novel.comment.created":
		table = "comments"
	case "novel.review.created":
		table = "reviews"
	case "novel.forum.thread.created":
		table = "forum_threads"
	case "novel.forum.reply.created":
		table = "forum_replies"
	default:
		return nil
	}

	query := fmt.Sprintf("SELECT user_id::text FROM %s WHERE id = $1", table)
	err := m.db.QueryRow(ctx, query, event.EntityID).Scan(&userID)
	if err != nil {
		return err
	}

	window := time.Now().Add(-5 * time.Minute)
	countQuery := fmt.Sprintf("SELECT COUNT(*) FROM %s WHERE user_id = $1 AND created_at > $2", table)

	var count int64
	err = m.db.QueryRow(ctx, countQuery, userID, window).Scan(&count)
	if err != nil {
		return err
	}

	if count > 20 {
		log.Printf("moderation: user %s rate abuse detected (%d posts in 5min on %s)", userID, count, table)
		_, err = m.db.Exec(ctx,
			"UPDATE users SET is_shadowbanned = TRUE WHERE id = $1", userID)
		return err
	}

	return nil
}

func (m *ModerationWorker) checkReportThreshold(ctx context.Context, entityID string) error {
	var count int64
	err := m.db.QueryRow(ctx,
		"SELECT COUNT(*) FROM reports WHERE entity_id = $1 AND status = 'pending'", entityID,
	).Scan(&count)
	if err != nil {
		return err
	}

	if count >= 5 {
		log.Printf("moderation: entity %s has %d pending reports, auto-hiding", entityID, count)
	}

	return nil
}
