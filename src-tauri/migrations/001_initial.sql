-- 설정 (단일 행)
CREATE TABLE IF NOT EXISTS settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    school_name TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 교사 계정
CREATE TABLE IF NOT EXISTS teachers (
    id INTEGER PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('admin', 'teacher')) DEFAULT 'teacher',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 교사 인증 토큰 (쿠키 기반 세션)
CREATE TABLE IF NOT EXISTS auth_tokens (
    id INTEGER PRIMARY KEY,
    teacher_id INTEGER NOT NULL REFERENCES teachers(id) ON DELETE CASCADE,
    token TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL
);

-- 수업 (과목 × 학생 그룹 단위. 예: "정보과학 1반")
-- 구 divisions 테이블. admin은 전체 접근, 일반교사는 teacher_classes로 제한.
CREATE TABLE IF NOT EXISTS classes (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,                    -- 표시명. 예: "정보과학 1반"
    subject TEXT NOT NULL DEFAULT '',      -- 과목명. 예: "정보과학"
    grade INTEGER NOT NULL DEFAULT 0,      -- 학년. 예: 3
    class_no INTEGER NOT NULL DEFAULT 0,   -- 반. 예: 1
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 교사-수업 M:N (일반교사 전용, admin은 전체 접근)
CREATE TABLE IF NOT EXISTS teacher_classes (
    teacher_id INTEGER NOT NULL REFERENCES teachers(id) ON DELETE CASCADE,
    class_id INTEGER NOT NULL REFERENCES classes(id) ON DELETE CASCADE,
    PRIMARY KEY (teacher_id, class_id)
);

-- 학생 (학번 + 비밀번호 로그인)
CREATE TABLE IF NOT EXISTS students (
    id INTEGER PRIMARY KEY,
    class_id INTEGER NOT NULL REFERENCES classes(id),
    student_number TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    password_hash TEXT NOT NULL DEFAULT '',
    password_reset_required INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 학생 세션 (쿠키 기반, cc_student)
CREATE TABLE IF NOT EXISTS student_sessions (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    token TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL
);

-- 문제 은행 (전역 공유. 수업에 속하지 않음)
CREATE TABLE IF NOT EXISTS problems (
    id INTEGER PRIMARY KEY,
    type INTEGER NOT NULL CHECK (type BETWEEN 1 AND 5),
    -- 1=OX  2=단답형  3=객관식  4=서술형  5=코딩
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    type_config TEXT NOT NULL DEFAULT '{}',
    is_structure_check INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 차시 (수업 소속. is_released로 학생 공개 여부 관리)
CREATE TABLE IF NOT EXISTS lessons (
    id INTEGER PRIMARY KEY,
    class_id INTEGER NOT NULL REFERENCES classes(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    order_no INTEGER NOT NULL DEFAULT 0,
    is_released INTEGER NOT NULL DEFAULT 0,
    released_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 차시-문항 배정 M:N
CREATE TABLE IF NOT EXISTS lesson_problems (
    id INTEGER PRIMARY KEY,
    lesson_id INTEGER NOT NULL REFERENCES lessons(id) ON DELETE CASCADE,
    problem_id INTEGER NOT NULL REFERENCES problems(id) ON DELETE RESTRICT,
    order_no INTEGER NOT NULL DEFAULT 0,
    UNIQUE (lesson_id, problem_id)
);

-- 수행평가 (수업 소속)
CREATE TABLE IF NOT EXISTS assessments (
    id INTEGER PRIMARY KEY,
    class_id INTEGER NOT NULL REFERENCES classes(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 수행평가-문항 배정 M:N (배점 포함)
CREATE TABLE IF NOT EXISTS assessment_problems (
    id INTEGER PRIMARY KEY,
    assessment_id INTEGER NOT NULL REFERENCES assessments(id) ON DELETE CASCADE,
    problem_id INTEGER NOT NULL REFERENCES problems(id) ON DELETE RESTRICT,
    order_no INTEGER NOT NULL DEFAULT 0,
    score INTEGER NOT NULL DEFAULT 0,
    UNIQUE (assessment_id, problem_id)
);

-- 수행평가 세션 (CREATED → LOBBY → RUNNING → CLOSED)
-- class_id는 assessment.class_id와 항상 동일. 직접 참조로 쿼리 편의 확보.
CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY,
    assessment_id INTEGER NOT NULL REFERENCES assessments(id),
    class_id INTEGER NOT NULL REFERENCES classes(id),
    status TEXT NOT NULL CHECK (status IN ('CREATED', 'LOBBY', 'RUNNING', 'CLOSED')) DEFAULT 'CREATED',
    target_type TEXT NOT NULL CHECK (target_type IN ('ALL', 'INDIVIDUAL')) DEFAULT 'ALL',
    time_limit_min INTEGER,
    start_at TEXT,
    end_at TEXT,
    is_paused INTEGER NOT NULL DEFAULT 0,
    paused_at TEXT,
    is_result_released INTEGER NOT NULL DEFAULT 0,
    parent_session_id INTEGER REFERENCES sessions(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 세션 개별 대상 (INDIVIDUAL 타입일 때만)
CREATE TABLE IF NOT EXISTS session_targets (
    id INTEGER PRIMARY KEY,
    session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    UNIQUE (session_id, student_id)
);

-- 제출 기록 (session_id=NULL이면 평소 수업 중 제출)
CREATE TABLE IF NOT EXISTS submissions (
    id INTEGER PRIMARY KEY,
    problem_id INTEGER NOT NULL REFERENCES problems(id),
    student_id INTEGER NOT NULL REFERENCES students(id),
    session_id INTEGER REFERENCES sessions(id),
    submission_no INTEGER NOT NULL DEFAULT 1,
    is_latest INTEGER NOT NULL DEFAULT 1,
    language TEXT,
    content TEXT NOT NULL DEFAULT '',
    verdict TEXT,
    score INTEGER,
    judged_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 출결 하트비트 (실시간 접속 위젯 데이터)
CREATE TABLE IF NOT EXISTS attendance_heartbeats (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    context_type TEXT NOT NULL CHECK (context_type IN ('lesson', 'session')),
    context_id INTEGER NOT NULL,
    joined_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_seen_at TEXT NOT NULL DEFAULT (datetime('now')),
    is_late INTEGER NOT NULL DEFAULT 0
);

-- 감사 로그
CREATE TABLE IF NOT EXISTS audit_logs (
    id INTEGER PRIMARY KEY,
    actor_teacher_id INTEGER REFERENCES teachers(id),
    action_type TEXT NOT NULL,
    target_type TEXT,
    target_id INTEGER,
    detail TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 인덱스
CREATE INDEX IF NOT EXISTS idx_auth_tokens_token ON auth_tokens(token);
CREATE INDEX IF NOT EXISTS idx_auth_tokens_teacher ON auth_tokens(teacher_id);
CREATE INDEX IF NOT EXISTS idx_student_sessions_token ON student_sessions(token);
CREATE INDEX IF NOT EXISTS idx_student_sessions_student ON student_sessions(student_id);
CREATE INDEX IF NOT EXISTS idx_students_class ON students(class_id);
CREATE INDEX IF NOT EXISTS idx_lessons_class_order ON lessons(class_id, order_no);
CREATE INDEX IF NOT EXISTS idx_lesson_problems_order ON lesson_problems(lesson_id, order_no);
CREATE INDEX IF NOT EXISTS idx_assessments_class ON assessments(class_id);
CREATE INDEX IF NOT EXISTS idx_sessions_class ON sessions(class_id, status);
CREATE INDEX IF NOT EXISTS idx_submissions_student_session ON submissions(student_id, session_id);
CREATE INDEX IF NOT EXISTS idx_submissions_is_latest ON submissions(problem_id, student_id, is_latest);
CREATE INDEX IF NOT EXISTS idx_attendance_context ON attendance_heartbeats(context_type, context_id);
