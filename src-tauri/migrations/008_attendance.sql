-- ── 출석 체크 이벤트 ──────────────────────────────────────────────────────────
-- 교사가 출석 체크 버튼을 누른 순간을 박제.
-- 하나의 수업에 하루 여러 번 체크 가능 (2교시 수업 등).
-- lesson과 연결하지 않음 — 교사가 매 차시 lesson을 만들지 않을 수 있음.
-- note: 진도 메모, 특이사항 등 자유 기록. 예: "변수와 연산자 2/3차시"

CREATE TABLE IF NOT EXISTS attendance_checks (
    id         INTEGER PRIMARY KEY,
    class_id   INTEGER NOT NULL REFERENCES classes(id) ON DELETE CASCADE,
    checked_at TEXT NOT NULL DEFAULT (datetime('now')),
    note       TEXT NOT NULL DEFAULT ''
);

-- ── 출결 기록 ─────────────────────────────────────────────────────────────────
-- 체크 이벤트별 학생 출결 상태.
-- is_present: 1=출석, 0=결석.
-- 초기값: 체크 시점에 로그인 중인 학생 → 1, 나머지 → 0 (백엔드 자동 처리).
-- 이후 교사가 개별 수정 가능 (지각 학생 결석→출석 전환 등).

CREATE TABLE IF NOT EXISTS attendance_records (
    id         INTEGER PRIMARY KEY,
    check_id   INTEGER NOT NULL REFERENCES attendance_checks(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    is_present INTEGER NOT NULL DEFAULT 0 CHECK (is_present IN (0, 1)),
    UNIQUE (check_id, student_id)
);

-- ── 인덱스 ────────────────────────────────────────────────────────────────────

CREATE INDEX IF NOT EXISTS idx_attendance_checks_class  ON attendance_checks(class_id, checked_at);
CREATE INDEX IF NOT EXISTS idx_attendance_records_check ON attendance_records(check_id);
CREATE INDEX IF NOT EXISTS idx_attendance_records_student ON attendance_records(student_id);
