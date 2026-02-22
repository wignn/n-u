# Novel Platform Architecture

## System Overview

```
Client -> Nginx (TLS + Rate Limit + Cache) -> Rust API (Axum) -> PostgreSQL
                                                               -> Redis (cache/session)
                                                               -> NATS JetStream (events)
                                                                     |
                                              Go Workers <-----------+
                                                |-> Search Indexer -> Meilisearch
                                                |-> Notification Worker -> PostgreSQL
                                                |-> Cache Invalidator -> Redis
                                                |-> Moderation Worker -> PostgreSQL
```

## Data Flow

1. Client sends HTTPS request to Nginx
2. Nginx terminates TLS, applies rate limits, checks proxy cache
3. Request forwarded to Rust API on port 3000
4. API reads/writes PostgreSQL as source of truth
5. API inserts outbox event in same transaction
6. Background poller publishes outbox events to NATS JetStream
7. Go workers consume events, re-fetch from PostgreSQL, update search/cache/notifications

## Event System

### Topic Convention

```
novel.{domain}.{action}
```

### Payload Format

```json
{
  "entity_id": "uuid",
  "event_type": "novel.novel.created",
  "timestamp": "2026-02-23T04:30:00Z"
}
```

### Transactional Outbox Pattern

```
BEGIN TRANSACTION
  INSERT business data
  INSERT INTO outbox_events (entity_id, event_type)
COMMIT

Background poller:
  SELECT FROM outbox_events WHERE published = FALSE
  PUBLISH to NATS JetStream
  UPDATE outbox_events SET published = TRUE
```

## Redis Cache Key Strategy

| Pattern | TTL | Purpose |
|---------|-----|---------|
| `novel:novel:{id}:detail` | 5min | Novel detail cache |
| `novel:novel:{id}:chapters` | 5min | Chapter list cache |
| `novel:novel:{id}:rating` | 5min | Rating aggregate |
| `novel:novel:trending` | 1min | Trending sorted set |
| `novel:novel:latest` | 1min | Latest sorted set |
| `novel:session:{id}` | 24h | User session |
| `novel:ratelimit:{ip}:{endpoint}` | 1min | Rate limit counter |
| `novel:counter:novel:{id}:views` | none | View counter |

## Rate Limiting (Nginx)

| Zone | Rate | Endpoints |
|------|------|-----------|
| login | 5r/s burst=10 | `/api/v1/users/login`, `/register` |
| posting | 2r/s burst=5 | POST `/api/v1/novels`, `/forums/threads` |
| comments | 5r/s burst=10 | `/api/v1/comments` |
| general | 30r/s burst=20 | All other endpoints |

## Project Structure

```
novel-update/
  docker-compose.yml
  .env.example
  nginx/
    nginx.conf
    ssl/
  novel-api/                          (Rust / Axum)
    Cargo.toml
    Dockerfile
    migrations/
      001_initial.sql
    src/
      main.rs
      lib.rs
      config.rs
      app_state.rs
      error.rs
      db/mod.rs
      middleware/
        mod.rs, auth.rs, request_id.rs
      events/
        mod.rs, publisher.rs, outbox.rs
      modules/
        mod.rs
        users/      (mod, routes, handlers, models, repository)
        novels/     (mod, routes, handlers, models, repository)
        chapters/   (mod, routes, handlers, models, repository)
        comments/   (mod, routes, handlers, models, repository)
        reviews/    (mod, routes, handlers, models, repository)
        forums/     (mod, routes, handlers, models, repository)
        bookmarks/  (mod, routes, handlers, models, repository)
        notifications/ (mod, routes, handlers, models, repository)
        admin/      (mod, routes, handlers, models, repository)
        search/     (mod, routes, handlers)
  novel-workers/                      (Go)
    go.mod
    Dockerfile
    cmd/workers/main.go
    internal/
      config/config.go
      db/postgres.go
      nats/consumer.go
      models/events.go
      workers/
        search_indexer.go
        notification.go
        cache_invalidator.go
        moderation.go
```

## API Endpoints

### Public
- `POST /api/v1/users/register`
- `POST /api/v1/users/login`
- `GET /api/v1/users/{id}/profile`
- `GET /api/v1/novels`
- `GET /api/v1/novels/{id}`
- `GET /api/v1/novels/by-slug/{slug}`
- `GET /api/v1/novels/{id}/chapters`
- `GET /api/v1/novels/{id}/reviews`
- `GET /api/v1/search?q=&index=`
- `GET /api/v1/forums/categories`
- `GET /api/v1/forums/categories/{id}/threads`
- `GET /api/v1/forums/threads/{id}`
- `GET /api/v1/forums/threads/{id}/replies`
- `GET /api/v1/comments?entity_type=&entity_id=`

### Authenticated
- `GET /api/v1/users/me`
- `PUT /api/v1/users/me/profile`
- `POST /api/v1/novels`
- `PUT /api/v1/novels/{id}`
- `DELETE /api/v1/novels/{id}`
- `POST /api/v1/novels/{id}/chapters`
- `DELETE /api/v1/novels/{id}/chapters/{chapter_id}`
- `POST /api/v1/novels/{id}/reviews`
- `PUT /api/v1/novels/{id}/reviews/{review_id}`
- `DELETE /api/v1/novels/{id}/reviews/{review_id}`
- `POST /api/v1/comments`
- `DELETE /api/v1/comments/{id}`
- `POST /api/v1/forums/threads`
- `POST /api/v1/forums/threads/{id}/replies`
- `GET/POST/DELETE /api/v1/bookmarks`
- `GET/PUT /api/v1/history`
- `GET /api/v1/notifications`
- `GET /api/v1/notifications/unread-count`
- `PUT /api/v1/notifications/{id}/read`
- `PUT /api/v1/notifications/read-all`

### Admin/Moderator
- `POST /api/v1/admin/reports`
- `GET /api/v1/admin/reports`
- `PUT /api/v1/admin/reports/{id}/resolve`
- `PUT /api/v1/admin/shadowban`
- `PUT /api/v1/admin/user-status`
- `GET /api/v1/admin/stats`

## Deployment

```bash
cp .env.example .env
# Edit .env with production values

# Generate self-signed certs for dev
cd nginx/ssl
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
    -keyout privkey.pem -out fullchain.pem -subj "/CN=localhost"
cd ../..

docker compose up -d
```
