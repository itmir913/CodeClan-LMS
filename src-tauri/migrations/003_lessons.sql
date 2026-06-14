-- ── 차시 ──────────────────────────────────────────────────────────────────────
-- 학교 공유 자원. 특정 수업에 종속되지 않음.
-- 같은 교과 교사들이 차시를 공유할 수 있다.

CREATE TABLE IF NOT EXISTS lessons (
    id          INTEGER PRIMARY KEY,
    created_by  INTEGER REFERENCES teachers(id) ON DELETE SET NULL,
    subject_id  INTEGER NOT NULL REFERENCES subjects(id),
    title       TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── 수업-차시 배정 ────────────────────────────────────────────────────────────
-- 수업별 공개 여부와 순서는 여기서 관리.
-- 같은 차시를 여러 수업에 배정 가능.

CREATE TABLE IF NOT EXISTS class_lessons (
    id          INTEGER PRIMARY KEY,
    class_id    INTEGER NOT NULL REFERENCES classes(id) ON DELETE CASCADE,
    lesson_id   INTEGER NOT NULL REFERENCES lessons(id) ON DELETE CASCADE,
    order_no    INTEGER NOT NULL DEFAULT 0 CHECK (order_no >= 0),
    is_released INTEGER NOT NULL DEFAULT 0 CHECK (is_released IN (0, 1)),
    released_at TEXT,
    UNIQUE (class_id, lesson_id)
);

-- ── 인덱스 ────────────────────────────────────────────────────────────────────

CREATE INDEX IF NOT EXISTS idx_lessons_subject     ON lessons(subject_id);
CREATE INDEX IF NOT EXISTS idx_class_lessons_class ON class_lessons(class_id, order_no);
