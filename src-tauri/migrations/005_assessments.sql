-- ── 수행평가 ──────────────────────────────────────────────────────────────────
-- 학교 공유 자원. lessons와 동일한 패턴.
-- 같은 교과 교사끼리 수행평가를 공유할 수 있다.
-- subject_id: NOT NULL. 관리자 초기 세팅에서 교과 생성 후 사용.
-- is_draft: 1=임시저장, 0=공개 (수업 배정 및 세션 생성 가능)

CREATE TABLE IF NOT EXISTS assessments (
    id          INTEGER PRIMARY KEY,
    created_by  INTEGER REFERENCES teachers(id) ON DELETE SET NULL,
    subject_id  INTEGER NOT NULL REFERENCES subjects(id),
    title       TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    is_draft    INTEGER NOT NULL DEFAULT 1 CHECK (is_draft IN (0, 1)),
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── 수업-수행평가 배정 (M:N) ──────────────────────────────────────────────────
-- class_lessons와 동일한 패턴.
-- order_no: 수업 내 수행평가 순서.
-- is_announced: 학생에게 수행평가 존재를 사전 공지할지 여부 (시험 전).
-- is_result_released: 학생에게 결과를 공개할지 여부 (시험 후, 수행평가 단위).
--   여러 세션이 있어도 결과 공개는 수행평가 단위로 일괄 처리.
-- 실제 시험 실행은 exam_sessions(006)에서 관리.

CREATE TABLE IF NOT EXISTS class_assessments (
    id                 INTEGER PRIMARY KEY,
    class_id           INTEGER NOT NULL REFERENCES classes(id) ON DELETE CASCADE,
    assessment_id      INTEGER NOT NULL REFERENCES assessments(id) ON DELETE CASCADE,
    order_no           INTEGER NOT NULL DEFAULT 0,
    is_announced       INTEGER NOT NULL DEFAULT 0 CHECK (is_announced IN (0, 1)),
    is_result_released INTEGER NOT NULL DEFAULT 0 CHECK (is_result_released IN (0, 1)),
    UNIQUE (class_id, assessment_id)
);

-- ── 수행평가-문항 배정 (M:N) ──────────────────────────────────────────────────
-- score: 배점. 같은 문항도 수행평가마다 배점이 다를 수 있음.
-- 배정된 문항은 직접 삭제 불가 (RESTRICT).

CREATE TABLE IF NOT EXISTS assessment_problems (
    id            INTEGER PRIMARY KEY,
    assessment_id INTEGER NOT NULL REFERENCES assessments(id) ON DELETE CASCADE,
    problem_id    INTEGER NOT NULL REFERENCES problems(id) ON DELETE RESTRICT,
    order_no      INTEGER NOT NULL DEFAULT 0,
    score         INTEGER NOT NULL DEFAULT 0 CHECK (score >= 0),
    UNIQUE (assessment_id, problem_id)
);

-- ── 인덱스 ────────────────────────────────────────────────────────────────────

CREATE INDEX IF NOT EXISTS idx_assessments_subject      ON assessments(subject_id);
CREATE INDEX IF NOT EXISTS idx_assessments_created_by   ON assessments(created_by);
CREATE INDEX IF NOT EXISTS idx_assessments_draft        ON assessments(is_draft);
CREATE INDEX IF NOT EXISTS idx_class_assessments_class  ON class_assessments(class_id, order_no);
CREATE INDEX IF NOT EXISTS idx_assessment_problems      ON assessment_problems(assessment_id, order_no);
