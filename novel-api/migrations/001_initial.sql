CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TYPE user_role AS ENUM ('user', 'moderator', 'admin');
CREATE TYPE user_status AS ENUM ('active', 'suspended', 'banned');
CREATE TYPE novel_status AS ENUM ('draft', 'ongoing', 'completed', 'hiatus', 'dropped');
CREATE TYPE content_type AS ENUM ('novel', 'chapter', 'review', 'forum_thread', 'forum_reply');
CREATE TYPE report_status AS ENUM ('pending', 'reviewed', 'resolved', 'dismissed');
CREATE TYPE notification_type AS ENUM ('reply', 'follow', 'review', 'system', 'moderation');

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(100),
    avatar_url TEXT,
    bio TEXT,
    role user_role NOT NULL DEFAULT 'user',
    status user_status NOT NULL DEFAULT 'active',
    is_shadowbanned BOOLEAN NOT NULL DEFAULT FALSE,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE novels (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(500) NOT NULL,
    slug VARCHAR(500) NOT NULL UNIQUE,
    synopsis TEXT,
    cover_url TEXT,
    status novel_status NOT NULL DEFAULT 'draft',
    genres TEXT[] NOT NULL DEFAULT '{}',
    tags TEXT[] NOT NULL DEFAULT '{}',
    chapter_count INTEGER NOT NULL DEFAULT 0,
    view_count BIGINT NOT NULL DEFAULT 0,
    is_visible BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE chapters (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    novel_id UUID NOT NULL REFERENCES novels(id) ON DELETE CASCADE,
    chapter_number INTEGER NOT NULL,
    title VARCHAR(500),
    link TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (novel_id, chapter_number)
);

CREATE TABLE reviews (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    novel_id UUID NOT NULL REFERENCES novels(id) ON DELETE CASCADE,
    rating SMALLINT NOT NULL CHECK (rating >= 1 AND rating <= 5),
    title VARCHAR(300),
    body TEXT NOT NULL,
    upvote_count INTEGER NOT NULL DEFAULT 0,
    is_visible BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, novel_id)
);

CREATE TABLE novel_ratings_agg (
    novel_id UUID PRIMARY KEY REFERENCES novels(id) ON DELETE CASCADE,
    avg_rating NUMERIC(3, 2) NOT NULL DEFAULT 0,
    rating_count INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE comments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    entity_type content_type NOT NULL,
    entity_id UUID NOT NULL,
    parent_id UUID REFERENCES comments(id) ON DELETE CASCADE,
    depth SMALLINT NOT NULL DEFAULT 0 CHECK (depth <= 5),
    body TEXT NOT NULL,
    upvote_count INTEGER NOT NULL DEFAULT 0,
    is_visible BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE forum_categories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(200) NOT NULL UNIQUE,
    slug VARCHAR(200) NOT NULL UNIQUE,
    description TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_locked BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE forum_threads (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    category_id UUID NOT NULL REFERENCES forum_categories(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(500) NOT NULL,
    slug VARCHAR(500) NOT NULL,
    body TEXT NOT NULL,
    is_pinned BOOLEAN NOT NULL DEFAULT FALSE,
    is_locked BOOLEAN NOT NULL DEFAULT FALSE,
    is_visible BOOLEAN NOT NULL DEFAULT TRUE,
    reply_count INTEGER NOT NULL DEFAULT 0,
    last_reply_at TIMESTAMPTZ,
    view_count BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (category_id, slug)
);

CREATE TABLE forum_replies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    thread_id UUID NOT NULL REFERENCES forum_threads(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    body TEXT NOT NULL,
    is_visible BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE bookmarks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    novel_id UUID NOT NULL REFERENCES novels(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, novel_id)
);

CREATE TABLE reading_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    novel_id UUID NOT NULL REFERENCES novels(id) ON DELETE CASCADE,
    last_chapter_id UUID REFERENCES chapters(id) ON DELETE SET NULL,
    last_chapter_number INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, novel_id)
);

CREATE TABLE user_follows (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    follower_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    following_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (follower_id, following_id),
    CHECK (follower_id != following_id)
);

CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notification_type notification_type NOT NULL,
    title VARCHAR(500) NOT NULL,
    body TEXT,
    entity_type content_type,
    entity_id UUID,
    actor_id UUID REFERENCES users(id) ON DELETE SET NULL,
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE reports (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    reporter_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    entity_type content_type NOT NULL,
    entity_id UUID NOT NULL,
    reason VARCHAR(1000) NOT NULL,
    status report_status NOT NULL DEFAULT 'pending',
    moderator_id UUID REFERENCES users(id) ON DELETE SET NULL,
    moderator_note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ
);

CREATE TABLE outbox_events (
    id BIGSERIAL PRIMARY KEY,
    entity_id UUID NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published BOOLEAN NOT NULL DEFAULT FALSE,
    published_at TIMESTAMPTZ
);

CREATE INDEX idx_users_username ON users (username);
CREATE INDEX idx_users_email ON users (email);
CREATE INDEX idx_users_status ON users (status) WHERE status != 'active';
CREATE INDEX idx_users_not_shadowbanned ON users (id) WHERE is_shadowbanned = FALSE;

CREATE INDEX idx_novels_author ON novels (author_id);
CREATE INDEX idx_novels_slug ON novels (slug);
CREATE INDEX idx_novels_status ON novels (status);
CREATE INDEX idx_novels_created ON novels (created_at DESC);
CREATE INDEX idx_novels_view_count ON novels (view_count DESC);
CREATE INDEX idx_novels_visible ON novels (id) WHERE is_visible = TRUE;
CREATE INDEX idx_novels_tags ON novels USING GIN (tags);
CREATE INDEX idx_novels_genres ON novels USING GIN (genres);

CREATE INDEX idx_chapters_novel ON chapters (novel_id, chapter_number);
CREATE INDEX idx_chapters_created ON chapters (novel_id, created_at DESC);

CREATE INDEX idx_reviews_novel ON reviews (novel_id, created_at DESC);
CREATE INDEX idx_reviews_user ON reviews (user_id);
CREATE INDEX idx_reviews_rating ON reviews (novel_id, rating);

CREATE INDEX idx_comments_entity ON comments (entity_type, entity_id, created_at);
CREATE INDEX idx_comments_parent ON comments (parent_id) WHERE parent_id IS NOT NULL;
CREATE INDEX idx_comments_user ON comments (user_id);

CREATE INDEX idx_forum_threads_category ON forum_threads (category_id, created_at DESC);
CREATE INDEX idx_forum_threads_pinned ON forum_threads (category_id, is_pinned DESC, last_reply_at DESC NULLS LAST);
CREATE INDEX idx_forum_threads_user ON forum_threads (user_id);
CREATE INDEX idx_forum_threads_slug ON forum_threads (category_id, slug);

CREATE INDEX idx_forum_replies_thread ON forum_replies (thread_id, created_at);
CREATE INDEX idx_forum_replies_user ON forum_replies (user_id);

CREATE INDEX idx_bookmarks_user ON bookmarks (user_id, created_at DESC);
CREATE INDEX idx_bookmarks_novel ON bookmarks (novel_id);

CREATE INDEX idx_reading_history_user ON reading_history (user_id, updated_at DESC);

CREATE INDEX idx_user_follows_follower ON user_follows (follower_id);
CREATE INDEX idx_user_follows_following ON user_follows (following_id);

CREATE INDEX idx_notifications_user ON notifications (user_id, created_at DESC);
CREATE INDEX idx_notifications_unread ON notifications (user_id, is_read) WHERE is_read = FALSE;

CREATE INDEX idx_reports_status ON reports (status, created_at DESC);
CREATE INDEX idx_reports_entity ON reports (entity_type, entity_id);

CREATE INDEX idx_outbox_unpublished ON outbox_events (id) WHERE published = FALSE;

CREATE OR REPLACE FUNCTION update_novel_rating_agg()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO novel_ratings_agg (novel_id, avg_rating, rating_count, updated_at)
    SELECT
        COALESCE(NEW.novel_id, OLD.novel_id),
        COALESCE(AVG(rating), 0),
        COUNT(*),
        NOW()
    FROM reviews
    WHERE novel_id = COALESCE(NEW.novel_id, OLD.novel_id) AND is_visible = TRUE
    ON CONFLICT (novel_id) DO UPDATE SET
        avg_rating = EXCLUDED.avg_rating,
        rating_count = EXCLUDED.rating_count,
        updated_at = NOW();
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_review_insert
    AFTER INSERT ON reviews
    FOR EACH ROW EXECUTE FUNCTION update_novel_rating_agg();

CREATE TRIGGER trg_review_update
    AFTER UPDATE OF rating, is_visible ON reviews
    FOR EACH ROW EXECUTE FUNCTION update_novel_rating_agg();

CREATE TRIGGER trg_review_delete
    AFTER DELETE ON reviews
    FOR EACH ROW EXECUTE FUNCTION update_novel_rating_agg();

CREATE OR REPLACE FUNCTION update_chapter_count()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE novels SET chapter_count = (
        SELECT COUNT(*) FROM chapters WHERE novel_id = COALESCE(NEW.novel_id, OLD.novel_id)
    ), updated_at = NOW()
    WHERE id = COALESCE(NEW.novel_id, OLD.novel_id);
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_chapter_count_insert
    AFTER INSERT ON chapters
    FOR EACH ROW EXECUTE FUNCTION update_chapter_count();

CREATE TRIGGER trg_chapter_count_delete
    AFTER DELETE ON chapters
    FOR EACH ROW EXECUTE FUNCTION update_chapter_count();

CREATE OR REPLACE FUNCTION update_reply_count()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE forum_threads SET
        reply_count = (SELECT COUNT(*) FROM forum_replies WHERE thread_id = COALESCE(NEW.thread_id, OLD.thread_id)),
        last_reply_at = NOW(),
        updated_at = NOW()
    WHERE id = COALESCE(NEW.thread_id, OLD.thread_id);
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_reply_count_insert
    AFTER INSERT ON forum_replies
    FOR EACH ROW EXECUTE FUNCTION update_reply_count();

CREATE TRIGGER trg_reply_count_delete
    AFTER DELETE ON forum_replies
    FOR EACH ROW EXECUTE FUNCTION update_reply_count();

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_users_updated_at BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER trg_novels_updated_at BEFORE UPDATE ON novels FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER trg_chapters_updated_at BEFORE UPDATE ON chapters FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER trg_reviews_updated_at BEFORE UPDATE ON reviews FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER trg_comments_updated_at BEFORE UPDATE ON comments FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER trg_forum_threads_updated_at BEFORE UPDATE ON forum_threads FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER trg_forum_replies_updated_at BEFORE UPDATE ON forum_replies FOR EACH ROW EXECUTE FUNCTION set_updated_at();
