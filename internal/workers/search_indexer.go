package workers

import (
	"context"
	"encoding/json"
	"fmt"
	"log"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/meilisearch/meilisearch-go"
	"github.com/nats-io/nats.go"

	"github.com/novel-platform/novel-workers/internal/models"
)

type SearchIndexer struct {
	db     *pgxpool.Pool
	search meilisearch.ServiceManager
}

func NewSearchIndexer(db *pgxpool.Pool, search meilisearch.ServiceManager) *SearchIndexer {
	return &SearchIndexer{db: db, search: search}
}

func (s *SearchIndexer) Start(ctx context.Context, js nats.JetStreamContext) error {
	sub, err := js.QueueSubscribe("novel.novel.*", "search-indexer", func(msg *nats.Msg) {
		s.handleNovelEvent(ctx, msg)
	}, nats.Durable("search-indexer"), nats.ManualAck())
	if err != nil {
		return fmt.Errorf("subscribe novel events: %w", err)
	}

	sub2, err := js.QueueSubscribe("novel.forum.thread.*", "search-indexer", func(msg *nats.Msg) {
		s.handleForumEvent(ctx, msg)
	}, nats.Durable("search-indexer-forum"), nats.ManualAck())
	if err != nil {
		return fmt.Errorf("subscribe forum events: %w", err)
	}

	log.Printf("Search indexer started")

	<-ctx.Done()
	sub.Unsubscribe()
	sub2.Unsubscribe()
	return nil
}

func (s *SearchIndexer) handleNovelEvent(ctx context.Context, msg *nats.Msg) {
	var event models.EventPayload
	if err := json.Unmarshal(msg.Data, &event); err != nil {
		log.Printf("search indexer: unmarshal error: %v", err)
		msg.Nak()
		return
	}

	switch event.EventType {
	case "novel.novel.created", "novel.novel.updated":
		if err := s.indexNovel(ctx, event.EntityID); err != nil {
			log.Printf("search indexer: index novel error: %v", err)
			msg.Nak()
			return
		}
	case "novel.novel.deleted":
		if _, err := s.search.Index("novels").DeleteDocument(event.EntityID); err != nil {
			log.Printf("search indexer: delete novel error: %v", err)
			msg.Nak()
			return
		}
	}

	msg.Ack()
}

func (s *SearchIndexer) handleForumEvent(ctx context.Context, msg *nats.Msg) {
	var event models.EventPayload
	if err := json.Unmarshal(msg.Data, &event); err != nil {
		log.Printf("search indexer: unmarshal error: %v", err)
		msg.Nak()
		return
	}

	if event.EventType == "novel.forum.thread.created" {
		if err := s.indexForumThread(ctx, event.EntityID); err != nil {
			log.Printf("search indexer: index forum thread error: %v", err)
			msg.Nak()
			return
		}
	}

	msg.Ack()
}

func (s *SearchIndexer) indexNovel(ctx context.Context, entityID string) error {
	var doc map[string]any
	err := s.db.QueryRow(ctx,
		`SELECT json_build_object(
			'id', n.id,
			'title', n.title,
			'synopsis', n.synopsis,
			'genres', n.genres,
			'tags', n.tags,
			'status', n.status,
			'chapter_count', n.chapter_count,
			'author_username', u.username,
			'created_at', n.created_at
		) FROM novels n JOIN users u ON n.author_id = u.id WHERE n.id = $1`, entityID,
	).Scan(&doc)
	if err != nil {
		return fmt.Errorf("fetch novel: %w", err)
	}

	if _, err := s.search.Index("novels").AddDocuments([]map[string]any{doc}, "id"); err != nil {
		return fmt.Errorf("index novel: %w", err)
	}

	return nil
}

func (s *SearchIndexer) indexForumThread(ctx context.Context, entityID string) error {
	var doc map[string]any
	err := s.db.QueryRow(ctx,
		`SELECT json_build_object(
			'id', t.id,
			'title', t.title,
			'body', t.body,
			'category_id', t.category_id,
			'author_username', u.username,
			'created_at', t.created_at
		) FROM forum_threads t JOIN users u ON t.user_id = u.id WHERE t.id = $1`, entityID,
	).Scan(&doc)
	if err != nil {
		return fmt.Errorf("fetch thread: %w", err)
	}

	if _, err := s.search.Index("forum_threads").AddDocuments([]map[string]any{doc}, "id"); err != nil {
		return fmt.Errorf("index thread: %w", err)
	}

	return nil
}
