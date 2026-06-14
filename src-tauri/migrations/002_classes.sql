-- ── 수업 ──────────────────────────────────────────────────────────────────────
-- 교사가 자유롭게 이름 지정. 예: "정보과학 1-1"
-- subject_id: nullable. 과목 미지정 수업도 허용.

CREATE TABLE IF NOT EXISTS classes (
    id         INTEGER PRIMARY KEY,
    teacher_id INTEGER NOT NULL REFERENCES teachers(id),
    subject_id INTEGER REFERENCES subjects(id),
    name       TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── 수업-학생 수강 관계 ───────────────────────────────────────────────────────
-- 같은 학생이 여러 수업에 등록 가능.

CREATE TABLE IF NOT EXISTS class_students (
    class_id   INTEGER NOT NULL REFERENCES classes(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    PRIMARY KEY (class_id, student_id)
);

-- ── 지원 언어 ─────────────────────────────────────────────────────────────────
-- is_enabled: admin이 전역 on/off.
-- 표시 이름은 프론트엔드 i18n에서 처리 ($t('language.python') 등).

CREATE TABLE IF NOT EXISTS languages (
    id         INTEGER PRIMARY KEY,
    slug       TEXT NOT NULL UNIQUE,
    is_enabled INTEGER NOT NULL DEFAULT 1
);

-- ── 수업별 허용 언어 ──────────────────────────────────────────────────────────
-- 비어있으면 전역 활성 언어 전체 허용.
-- 전역 비활성 언어는 수업 설정과 무관하게 불가.

CREATE TABLE IF NOT EXISTS class_allowed_languages (
    class_id    INTEGER NOT NULL REFERENCES classes(id) ON DELETE CASCADE,
    language_id INTEGER NOT NULL REFERENCES languages(id) ON DELETE CASCADE,
    PRIMARY KEY (class_id, language_id)
);

-- ── 인덱스 ────────────────────────────────────────────────────────────────────

CREATE INDEX IF NOT EXISTS idx_classes_teacher        ON classes(teacher_id);
CREATE INDEX IF NOT EXISTS idx_classes_subject        ON classes(subject_id);
CREATE INDEX IF NOT EXISTS idx_class_students_student ON class_students(student_id);
