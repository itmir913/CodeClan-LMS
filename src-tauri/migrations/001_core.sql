-- ── 앱 설정 ──────────────────────────────────────────────────────────────────
-- 키-값 형태. 항목 추가 시 스키마 변경 불필요.

CREATE TABLE IF NOT EXISTS app_configs (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- ── 교과 ──────────────────────────────────────────────────────────────────────
-- 사용자 입력 데이터. i18n 대상 아님.

CREATE TABLE IF NOT EXISTS subjects (
    id   INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

-- ── 교사 ──────────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS teachers (
    id            INTEGER PRIMARY KEY,
    username      TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    name          TEXT NOT NULL,
    role          TEXT NOT NULL CHECK (role IN ('admin', 'teacher')) DEFAULT 'teacher',
    created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── 학생 ──────────────────────────────────────────────────────────────────────
-- username 자동생성 규칙: {grade}{class_no:02d}{number:02d} (예: 30102)
-- 수업(class)과 독립 엔티티. 수강 관계는 class_students.

CREATE TABLE IF NOT EXISTS students (
    id                      INTEGER PRIMARY KEY,
    username                TEXT NOT NULL UNIQUE,
    name                    TEXT NOT NULL,
    grade                   INTEGER NOT NULL,
    class_no                INTEGER NOT NULL,
    number                  INTEGER NOT NULL,
    password_hash           TEXT NOT NULL DEFAULT '',
    password_reset_required INTEGER NOT NULL DEFAULT 1 CHECK (password_reset_required IN (0, 1)),
    created_at              TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (grade, class_no, number)
);

-- ── 통합 인증 세션 ────────────────────────────────────────────────────────────
-- 교사·학생 공용. 쿠키명: cc_session
-- teacher_id / student_id 중 정확히 하나만 채워진다.

CREATE TABLE IF NOT EXISTS auth_sessions (
    id         INTEGER PRIMARY KEY,
    token      TEXT NOT NULL UNIQUE,
    teacher_id INTEGER REFERENCES teachers(id) ON DELETE CASCADE,
    student_id INTEGER REFERENCES students(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL,
    CHECK (
        (teacher_id IS NOT NULL AND student_id IS NULL) OR
        (teacher_id IS NULL     AND student_id IS NOT NULL)
    )
);

-- ── 사용자별 설정 ─────────────────────────────────────────────────────────────
-- 교사/학생 각자의 환경설정. 복합 PK로 (user, key) UNIQUE 보장.
-- 알려진 키: locale, dark_mode, editor_font_size

CREATE TABLE IF NOT EXISTS teacher_settings (
    teacher_id INTEGER NOT NULL REFERENCES teachers(id) ON DELETE CASCADE,
    key        TEXT NOT NULL,
    value      TEXT NOT NULL,
    PRIMARY KEY (teacher_id, key)
);

CREATE TABLE IF NOT EXISTS student_settings (
    student_id INTEGER NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    key        TEXT NOT NULL,
    value      TEXT NOT NULL,
    PRIMARY KEY (student_id, key)
);

-- ── 인덱스 ────────────────────────────────────────────────────────────────────

CREATE INDEX IF NOT EXISTS idx_auth_sessions_token   ON auth_sessions(token);
CREATE INDEX IF NOT EXISTS idx_auth_sessions_teacher ON auth_sessions(teacher_id);
CREATE INDEX IF NOT EXISTS idx_auth_sessions_student ON auth_sessions(student_id);
