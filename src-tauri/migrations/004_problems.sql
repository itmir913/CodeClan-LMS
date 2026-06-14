-- ── 문항 유형 ─────────────────────────────────────────────────────────────────
-- slug만 저장. 표시 이름은 i18n ($t('problem_type.short_answer') 등).
-- 초기값: short_answer, multiple_choice, code_submit, portfolio(예약)

CREATE TABLE IF NOT EXISTS problem_types (
    id   INTEGER PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE
);

-- ── 문항 (공통 베이스) ────────────────────────────────────────────────────────
-- 차시/수행평가 모두에서 참조하는 공통 자원.
-- description: 마크다운. 코드 제출형의 경우 문제 본문 역할.
-- comment: 교사 메모. 예: "Python 기준으로 설계됨"

CREATE TABLE IF NOT EXISTS problems (
    id          INTEGER PRIMARY KEY,
    type_id     INTEGER NOT NULL REFERENCES problem_types(id),
    created_by  INTEGER REFERENCES teachers(id) ON DELETE SET NULL,
    subject_id  INTEGER REFERENCES subjects(id),
    title       TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    comment     TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── 단답형 ────────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS problem_short_answers (
    problem_id     INTEGER PRIMARY KEY REFERENCES problems(id) ON DELETE CASCADE,
    answer         TEXT NOT NULL,
    case_sensitive INTEGER NOT NULL DEFAULT 0
);

-- ── 선다형 보기 ───────────────────────────────────────────────────────────────
-- 보기 개수 유동적 (2~n개).

CREATE TABLE IF NOT EXISTS problem_choices (
    id         INTEGER PRIMARY KEY,
    problem_id INTEGER NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
    order_no   INTEGER NOT NULL,
    content    TEXT NOT NULL,
    is_correct INTEGER NOT NULL DEFAULT 0,
    UNIQUE (problem_id, order_no)
);

-- ── 코드 제출형 설정 ──────────────────────────────────────────────────────────
-- input_format / output_format / constraints: 마크다운
-- show_io_on_fail: 실패 시 입력·기댓값·실제출력을 학생에게 공개할지 여부

CREATE TABLE IF NOT EXISTS problem_code_submits (
    problem_id      INTEGER PRIMARY KEY REFERENCES problems(id) ON DELETE CASCADE,
    input_format    TEXT NOT NULL DEFAULT '',
    output_format   TEXT NOT NULL DEFAULT '',
    constraints     TEXT NOT NULL DEFAULT '',
    time_limit_ms   INTEGER NOT NULL DEFAULT 1000,
    memory_limit_mb INTEGER NOT NULL DEFAULT 128,
    show_io_on_fail INTEGER NOT NULL DEFAULT 1
);

-- ── 테스트케이스 메타데이터 ───────────────────────────────────────────────────
-- 실제 내용은 파일: {data_dir}/problems/{problem_id}/{number}.in|out
-- is_sample=1: 문제 화면에 공개 (입출력 예시)
-- is_sample=0: 히든 케이스 (채점 전용)
-- 부분 점수 없음. 각 케이스는 통과/실패만.

CREATE TABLE IF NOT EXISTS problem_test_cases (
    id         INTEGER PRIMARY KEY,
    problem_id INTEGER NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
    number     INTEGER NOT NULL,
    is_sample  INTEGER NOT NULL DEFAULT 0,
    UNIQUE (problem_id, number)
);

-- ── 포트폴리오형 (추후 확장 예약) ────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS problem_portfolios (
    problem_id INTEGER PRIMARY KEY REFERENCES problems(id) ON DELETE CASCADE
);

-- ── 인덱스 ────────────────────────────────────────────────────────────────────

CREATE INDEX IF NOT EXISTS idx_problems_type       ON problems(type_id);
CREATE INDEX IF NOT EXISTS idx_problems_subject    ON problems(subject_id);
CREATE INDEX IF NOT EXISTS idx_problems_created_by ON problems(created_by);
CREATE INDEX IF NOT EXISTS idx_problem_choices     ON problem_choices(problem_id, order_no);
CREATE INDEX IF NOT EXISTS idx_test_cases_problem  ON problem_test_cases(problem_id, number);
