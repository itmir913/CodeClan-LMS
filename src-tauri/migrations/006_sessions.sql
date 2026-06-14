-- ── 시험 세션 ─────────────────────────────────────────────────────────────────
-- 수행평가의 실제 시험 실행 단위.
-- 같은 수행평가(class_assessment)에 여러 세션 가능 (재시험 등).
--
-- 상태 머신: WAITING → RUNNING ⇄ PAUSED → CLOSED
--   WAITING : 세션 생성됨. 학생 로그인 시 자동 대기실 입장.
--   RUNNING : 교사가 시작 버튼 → 타이머 가동.
--   PAUSED  : 교사 일시정지. 타이머 멈춤.
--   CLOSED  : 시간 종료(자동) 또는 교사 강제 종료.
--
-- 타이머 계산:
--   남은 시간 = time_limit_min * 60000 - elapsed_ms - (now - started_at)
--   PAUSED 시: elapsed_ms += (paused_at - started_at)
--   RUNNING 재개 시: started_at 갱신, elapsed_ms 누적값 유지
--
-- target_type:
--   ALL        : 수업 전체 학생 대상 (일반 시험)
--   INDIVIDUAL : 특정 학생만 대상 (재시험). session_targets에 목록 저장.

CREATE TABLE IF NOT EXISTS exam_sessions (
    id                   INTEGER PRIMARY KEY,
    class_assessment_id  INTEGER NOT NULL REFERENCES class_assessments(id) ON DELETE CASCADE,
    parent_session_id    INTEGER REFERENCES exam_sessions(id),
    status               TEXT NOT NULL DEFAULT 'WAITING'
                             CHECK (status IN ('WAITING', 'RUNNING', 'PAUSED', 'CLOSED')),
    target_type          TEXT NOT NULL DEFAULT 'ALL'
                             CHECK (target_type IN ('ALL', 'INDIVIDUAL')),
    time_limit_min       INTEGER,                          -- NULL이면 무제한
    started_at           TEXT,                             -- RUNNING 전환 시각 (마지막)
    elapsed_ms           INTEGER NOT NULL DEFAULT 0,       -- 현재 RUNNING 이전 누적 경과 ms
    paused_at            TEXT,                             -- 현재 PAUSED 전환 시각
    closed_at            TEXT,                             -- CLOSED 전환 시각
    created_at           TEXT NOT NULL DEFAULT (datetime('now')),
    CHECK (elapsed_ms >= 0)
);

-- ── 세션 개별 대상 ────────────────────────────────────────────────────────────
-- target_type = 'INDIVIDUAL'인 세션에만 사용.
-- 해당 학생만 세션에 입장 가능.

CREATE TABLE IF NOT EXISTS session_targets (
    session_id INTEGER NOT NULL REFERENCES exam_sessions(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    PRIMARY KEY (session_id, student_id)
);

-- ── 인덱스 ────────────────────────────────────────────────────────────────────

CREATE INDEX IF NOT EXISTS idx_exam_sessions_class_assessment ON exam_sessions(class_assessment_id);
CREATE INDEX IF NOT EXISTS idx_exam_sessions_status           ON exam_sessions(status);
CREATE INDEX IF NOT EXISTS idx_exam_sessions_parent           ON exam_sessions(parent_session_id);
CREATE INDEX IF NOT EXISTS idx_session_targets_student        ON session_targets(student_id);
