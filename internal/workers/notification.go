package workers

import (
	"context"
	"encoding/json"
	"fmt"
	"log"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/nats-io/nats.go"

	"github.com/novel-platform/novel-workers/internal/models"
)

type NotificationWorker struct {
	db *pgxpool.Pool
}

func NewNotificationWorker(db *pgxpool.Pool) *NotificationWorker {
	return &NotificationWorker{db: db}
}

func (n *NotificationWorker) Start(ctx context.Context, js nats.JetStreamContext) error {
	subjects := []string{
		"novel.comment.created",
		"novel.review.created",
		"novel.forum.reply.created",
		"novel.follow.created",
	}

	for _, subj := range subjects {
		s := subj
		_, err := js.QueueSubscribe(s, "notification-worker", func(msg *nats.Msg) {
			n.handleEvent(ctx, msg)
		}, nats.Durable("notif-"+s), nats.ManualAck())
		if err != nil {
			return fmt.Errorf("subscribe %s: %w", s, err)
		}
	}

	log.Printf("Notification worker started")

	<-ctx.Done()
	return nil
}

func (n *NotificationWorker) handleEvent(ctx context.Context, msg *nats.Msg) {
	var event models.EventPayload
	if err := json.Unmarshal(msg.Data, &event); err != nil {
		log.Printf("notification worker: unmarshal error: %v", err)
		msg.Nak()
		return
	}

	var err error
	switch event.EventType {
	case "novel.comment.created":
		err = n.notifyCommentReply(ctx, event.EntityID)
	case "novel.review.created":
		err = n.notifyNewReview(ctx, event.EntityID)
	case "novel.forum.reply.created":
		err = n.notifyForumReply(ctx, event.EntityID)
	case "novel.follow.created":
		err = n.notifyNewFollower(ctx, event.EntityID)
	}

	if err != nil {
		log.Printf("notification worker: %s error: %v", event.EventType, err)
		msg.Nak()
		return
	}

	msg.Ack()
}

func (n *NotificationWorker) notifyCommentReply(ctx context.Context, commentID string) error {
	var parentUserID, actorID *string
	var entityType string
	var entityID string

	err := n.db.QueryRow(ctx,
		`SELECT c.entity_type::text, c.entity_id::text, p.user_id::text, c.user_id::text
		 FROM comments c
		 LEFT JOIN comments p ON c.parent_id = p.id
		 WHERE c.id = $1 AND c.parent_id IS NOT NULL`, commentID,
	).Scan(&entityType, &entityID, &parentUserID, &actorID)
	if err != nil || parentUserID == nil {
		return err
	}

	if *parentUserID == *actorID {
		return nil
	}

	_, err = n.db.Exec(ctx,
		`INSERT INTO notifications (user_id, notification_type, title, entity_type, entity_id, actor_id)
		 VALUES ($1, 'reply', 'New reply to your comment', $2::content_type, $3, $4)`,
		parentUserID, entityType, entityID, actorID)
	return err
}

func (n *NotificationWorker) notifyNewReview(ctx context.Context, reviewID string) error {
	_, err := n.db.Exec(ctx,
		`INSERT INTO notifications (user_id, notification_type, title, entity_type, entity_id, actor_id)
		 SELECT n.author_id, 'review', 'New review on your novel', 'novel', r.novel_id, r.user_id
		 FROM reviews r JOIN novels n ON r.novel_id = n.id
		 WHERE r.id = $1 AND r.user_id != n.author_id`, reviewID)
	return err
}

func (n *NotificationWorker) notifyForumReply(ctx context.Context, replyID string) error {
	_, err := n.db.Exec(ctx,
		`INSERT INTO notifications (user_id, notification_type, title, entity_type, entity_id, actor_id)
		 SELECT t.user_id, 'reply', 'New reply in your thread', 'forum_thread', t.id, r.user_id
		 FROM forum_replies r JOIN forum_threads t ON r.thread_id = t.id
		 WHERE r.id = $1 AND r.user_id != t.user_id`, replyID)
	return err
}

func (n *NotificationWorker) notifyNewFollower(ctx context.Context, followID string) error {
	_, err := n.db.Exec(ctx,
		`INSERT INTO notifications (user_id, notification_type, title, actor_id)
		 SELECT following_id, 'follow', 'You have a new follower', follower_id
		 FROM user_follows WHERE id = $1`, followID)
	return err
}
