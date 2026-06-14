-- ── 차시 제출 (베이스) ────────────────────────────────────────────────────────
-- 문항 유형 공통 베이스. 유형별 상세는 확장 테이블에 저장.
-- is_correct: 단답형·선다형은 서버가 즉시 세팅. 코드형은 NULL (judge_status로 판단).
-- 같은 문항에 여러 번 제출 가능. 마지막 submitted_at 기준이 최종 답안.

CREATE TABLE IF NOT EXISTS lesson_submissions (
    id              INTEGER PRIMARY KEY,
    class_lesson_id INTEGER NOT NULL REFERENCES class_lessons(id) ON DELETE CASCADE,
    student_id      INTEGER NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    problem_id      INTEGER NOT NULL REFERENCES problems(id) ON DELETE RESTRICT,
    is_correct      INTEGER NOT NULL DEFAULT 0 CHECK (is_correct IN (0, 1)),
    submitted_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── 차시 단답형 제출 ──────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS lesson_submission_short_answers (
    submission_id INTEGER PRIMARY KEY REFERENCES lesson_submissions(id) ON DELETE CASCADE,
    answer_text   TEXT NOT NULL
);

-- ── 차시 선다형 선택 목록 ─────────────────────────────────────────────────────
-- 복수선택 지원. 선택한 보기 ID를 모두 저장.

CREATE TABLE IF NOT EXISTS lesson_submission_choices (
    submission_id INTEGER NOT NULL REFERENCES lesson_submissions(id) ON DELETE CASCADE,
    choice_id     INTEGER NOT NULL REFERENCES problem_choices(id) ON DELETE CASCADE,
    PRIMARY KEY (submission_id, choice_id)
);

-- ── 차시 코드 제출 ────────────────────────────────────────────────────────────
-- judge_message: COMPILE_ERROR 시 컴파일러 출력. 그 외 빈 문자열.

CREATE TABLE IF NOT EXISTS lesson_submission_codes (
    submission_id INTEGER PRIMARY KEY REFERENCES lesson_submissions(id) ON DELETE CASCADE,
    language_id   INTEGER NOT NULL REFERENCES languages(id),
    source_code   TEXT NOT NULL DEFAULT '',
    judge_status  TEXT NOT NULL DEFAULT 'PENDING'
                      CHECK (judge_status IN (
                          'PENDING', 'JUDGING', 'ACCEPTED',
                          'WRONG_ANSWER', 'TIME_LIMIT_EXCEEDED',
                          'MEMORY_LIMIT_EXCEEDED', 'RUNTIME_ERROR',
                          'COMPILE_ERROR', 'SYSTEM_ERROR'
                      )),
    judge_message TEXT NOT NULL DEFAULT ''
);

-- ── 차시 코드 채점 결과 ───────────────────────────────────────────────────────
-- 첫 번째 실패에서 중단. 실행된 케이스까지만 기록.
-- actual_output: show_io_on_fail=1 이면 프론트엔드에서 표시.

CREATE TABLE IF NOT EXISTS lesson_submission_test_results (
    id            INTEGER PRIMARY KEY,
    submission_id INTEGER NOT NULL REFERENCES lesson_submissions(id) ON DELETE CASCADE,
    test_case_id  INTEGER NOT NULL REFERENCES problem_test_cases(id) ON DELETE CASCADE,
    status        TEXT NOT NULL
                      CHECK (status IN (
                          'ACCEPTED', 'WRONG_ANSWER', 'TIME_LIMIT_EXCEEDED',
                          'MEMORY_LIMIT_EXCEEDED', 'RUNTIME_ERROR'
                      )),
    elapsed_ms    INTEGER NOT NULL DEFAULT 0 CHECK (elapsed_ms >= 0),
    memory_kb     INTEGER NOT NULL DEFAULT 0 CHECK (memory_kb >= 0),
    actual_output TEXT NOT NULL DEFAULT '',
    UNIQUE (submission_id, test_case_id)
);

-- ═════════════════════════════════════════════════════════════════════════════
-- ── 수행평가 제출 (베이스) ────────────────────────────────────────────────────
-- lesson_submissions와 동일한 패턴. 컨텍스트만 session_id로 다름.

CREATE TABLE IF NOT EXISTS assessment_submissions (
    id         INTEGER PRIMARY KEY,
    session_id INTEGER NOT NULL REFERENCES assessment_sessions(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    problem_id INTEGER NOT NULL REFERENCES problems(id) ON DELETE RESTRICT,
    is_correct   INTEGER NOT NULL DEFAULT 0 CHECK (is_correct IN (0, 1)),
    submitted_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── 수행평가 단답형 제출 ──────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS assessment_submission_short_answers (
    submission_id INTEGER PRIMARY KEY REFERENCES assessment_submissions(id) ON DELETE CASCADE,
    answer_text   TEXT NOT NULL
);

-- ── 수행평가 선다형 선택 목록 ─────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS assessment_submission_choices (
    submission_id INTEGER NOT NULL REFERENCES assessment_submissions(id) ON DELETE CASCADE,
    choice_id     INTEGER NOT NULL REFERENCES problem_choices(id) ON DELETE CASCADE,
    PRIMARY KEY (submission_id, choice_id)
);

-- ── 수행평가 코드 제출 ────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS assessment_submission_codes (
    submission_id INTEGER PRIMARY KEY REFERENCES assessment_submissions(id) ON DELETE CASCADE,
    language_id   INTEGER NOT NULL REFERENCES languages(id),
    source_code   TEXT NOT NULL DEFAULT '',
    judge_status  TEXT NOT NULL DEFAULT 'PENDING'
                      CHECK (judge_status IN (
                          'PENDING', 'JUDGING', 'ACCEPTED',
                          'WRONG_ANSWER', 'TIME_LIMIT_EXCEEDED',
                          'MEMORY_LIMIT_EXCEEDED', 'RUNTIME_ERROR',
                          'COMPILE_ERROR', 'SYSTEM_ERROR'
                      )),
    judge_message TEXT NOT NULL DEFAULT ''
);

-- ── 수행평가 코드 채점 결과 ───────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS assessment_submission_test_results (
    id            INTEGER PRIMARY KEY,
    submission_id INTEGER NOT NULL REFERENCES assessment_submissions(id) ON DELETE CASCADE,
    test_case_id  INTEGER NOT NULL REFERENCES problem_test_cases(id) ON DELETE CASCADE,
    status        TEXT NOT NULL
                      CHECK (status IN (
                          'ACCEPTED', 'WRONG_ANSWER', 'TIME_LIMIT_EXCEEDED',
                          'MEMORY_LIMIT_EXCEEDED', 'RUNTIME_ERROR'
                      )),
    elapsed_ms    INTEGER NOT NULL DEFAULT 0 CHECK (elapsed_ms >= 0),
    memory_kb     INTEGER NOT NULL DEFAULT 0 CHECK (memory_kb >= 0),
    actual_output TEXT NOT NULL DEFAULT '',
    UNIQUE (submission_id, test_case_id)
);

-- ── 인덱스 ────────────────────────────────────────────────────────────────────

CREATE INDEX IF NOT EXISTS idx_lesson_submissions_lesson   ON lesson_submissions(class_lesson_id);
CREATE INDEX IF NOT EXISTS idx_lesson_submissions_student  ON lesson_submissions(student_id, problem_id);
CREATE INDEX IF NOT EXISTS idx_lesson_sub_test_results     ON lesson_submission_test_results(submission_id);

CREATE INDEX IF NOT EXISTS idx_assessment_submissions_session  ON assessment_submissions(session_id);
CREATE INDEX IF NOT EXISTS idx_assessment_submissions_student  ON assessment_submissions(student_id, problem_id);
CREATE INDEX IF NOT EXISTS idx_assessment_sub_test_results     ON assessment_submission_test_results(submission_id);
