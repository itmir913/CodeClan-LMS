# CodeClan LMS — GLOBAL RULES

## ARCHITECTURE

### 백엔드 (Rust / Axum)
- **모든 DB 접근과 비즈니스 로직은 Rust 서버**에서 처리한다.
- 인증, 채점, 파일 I/O는 반드시 서버 핸들러에서 수행한다.
- SQLx 쿼리는 `src-tauri/src/server/routes/` 또는 `src-tauri/src/server/db/` 안에서만 작성한다.
- `AppState { db: SqlitePool }`를 통해 DB 풀을 공유한다.
- **`sqlx::query!` 매크로 사용 금지** — 컴파일 타임 DB 연결을 요구함. `sqlx::query()` / `sqlx::query_as::<_, Type>()` 런타임 쿼리만 사용한다.

### 프론트엔드 (Vue 3 / Pinia)
- **컴포넌트는 직접 API를 호출하지 않는다** — 반드시 Pinia store를 통한다.
- `frontend/src/api/client.ts`의 API 헬퍼 함수만 store에서 호출한다.
- 컴포넌트는 UI 렌더링과 사용자 이벤트 처리만 담당한다.
- Tauri `invoke()`는 현재 사용하지 않는다 (모든 기능은 Axum HTTP API로 처리).

### 상태 관리
- **Pinia store = 단일 진실의 원천(Single Source of Truth)**
- 서버에서 받은 데이터는 반드시 store에 저장 후 컴포넌트에서 참조한다.
- `stores/auth.ts`: 교사 세션 상태, 역할(admin/teacher), 학교 이름, 로그인 여부
- `stores/class.ts`: 수업 목록, 과목 목록, 수업 CRUD
- `stores/admin.ts`: 교사 계정 목록, 과목 관리 (admin 전용)
- 향후 추가: `stores/lesson.ts`, `stores/assessment.ts`, `stores/problem.ts`, `stores/student.ts`

---

## PRINCIPLES

- **Store = Single Source of Truth**: 서버 응답을 컴포넌트가 직접 보유하지 않는다.
- **Frontend = UI + 상태**: 비즈니스 규칙, 채점, 권한 판단은 모두 서버에 있다.
- **중복 로직 금지**: 서버가 처리하는 것을 프론트엔드에서 재구현하지 않는다.
- **단일 바이너리 원칙**: Tauri + Axum이 하나의 실행 파일 안에서 동작한다.

---

## UI / DESIGN RULES

- **CSS 변수(design token) 기반**: 색상, 간격, 반지름은 `frontend/src/assets/tokens.css`의 `:root` 변수를 참조한다. 인라인 하드코딩 금지.
- **Tailwind CSS v4** 사용. `@tailwindcss/vite` 플러그인 기반. 유틸리티 클래스 우선, `<style scoped>`는 Tailwind로 표현 불가한 경우에만 보조 사용.
- **CSS Cascade Layer 규칙**: `base.css`의 전역 `button`, `input` 등 폼 요소 기본 스타일은 반드시 `@layer base { }` 안에 선언한다. unlayered CSS는 `@layer utilities`(Tailwind)보다 cascade 우선순위가 높아 Tailwind 클래스가 무력화된다.
- **컴포넌트 스타일 3원칙**:
  1. 레이아웃·크기·간격 → Tailwind 클래스 (`h-12`, `px-4`, `rounded-lg`, `p-0`, `border-0` 등)
  2. 색상·그림자 등 CSS 변수 값 → `style="color: var(--color-accent)"` (CSS 변수만 허용)
  3. hover 등 상태에 CSS 변수 필요 시 → `<style scoped>` 사용
- **`style=""` 속성에 CSS 변수 외 값 금지**: `style="padding: 0"`, `style="border-radius: 8px"` 등 Tailwind로 표현 가능한 값을 inline으로 작성하지 않는다. 반드시 Tailwind 클래스로만 처리한다.
- **폰트**: 본문은 `Pretendard` (가변 폰트, npm 패키지 `pretendard` 사용), 코드·에디터 영역은 `Pretendard Mono` 또는 동급 고정폭 폰트. 외부 CDN 금지 — npm으로 번들에 포함.
- **다크모드 대응**: CSS 변수를 `:root`(라이트)와 `.dark` 또는 `[data-theme="dark"]` 두 벌로 정의한다. 색상은 반드시 시맨틱 변수(`--color-bg-primary`, `--color-text-primary` 등)로만 참조하여 테마 전환 시 자동 반영되도록 한다. 하드코딩 색상값 사용 금지.
- **최소 폰트 크기: `text-base` (16px)**. `text-sm`, `text-xs` 등 더 작은 클래스 사용 금지. 모든 텍스트는 `text-base` 이상이어야 한다.
- **아이콘: `@tabler/icons-vue` 패키지** 사용. 패키지가 없는 경우 SVG inline 대체.
- **홈 화면(교사·학생 공통)**: 사이드바 없음. 수업 카드 그리드만 표시.
- **수업 내부 화면(교사)**: 좌측 사이드바(차시·수행평가·세션·출석부·학생) + 우측 메인 콘텐츠.
- **시험 응시 화면(학생)**: 사이드바·네비게이션 완전 숨김. 타이머·문제·에디터·제출 버튼만.
- 로딩 상태와 에러 상태를 반드시 UI에 표시한다 (스피너, 에러 배너).
- 교사 화면: 최소 1024px 기준 / 학생 화면: 768px 이상 기준.
- `docs/mockups/*.html`이 있으면 레이아웃·색상 구조를 참고한다. 목업이 없어도 구현 가능.
- **모달 동작 규칙**: 모달 외부 클릭으로 닫히지 않는다. ESC 키로만 닫힌다. `@click.self` 기반 외부 클릭 닫기 구현 금지.

---

## CONVENTIONS

### Rust
- 함수명: `snake_case`
- 에러: `crate::error::ApiError` (IntoResponse impl). 핸들러 반환: `Result<Json<T>, ApiError>`.
- 인증: argon2 해시, `auth_tokens` 테이블 쿠키 세션(12시간). **JWT 사용 금지.**
- DB: `sqlx::query()` / `sqlx::query_as::<_, Type>()` 런타임 쿼리. `query!` 매크로 금지.
- 트랜잭션 필요 시: `db.begin().await?` → `tx.commit().await?`.

### TypeScript / Vue
- 컴포넌트 파일명: `PascalCase.vue`
- API 타입은 `frontend/src/api/client.ts` 내에 TypeScript `interface`로 선언.
- Composable: `useXxx.ts` (소문자 `use` 접두어)
- `<script setup lang="ts">` 형식 사용 (Options API 금지)
- **`any` 타입 사용 금지** — 명시적 interface로 대체.

### 버튼 가드 구조 (제출·삭제·저장 등 모든 비동기 액션)

모든 비동기 액션 버튼은 아래 4-레이어 구조를 반드시 준수한다.

```vue
<!-- ① 템플릿: disabled + 로딩 표시 -->
<button :disabled="isSubmitting" @click="onSubmit">
  <IconLoader2 v-if="isSubmitting" class="spin" />
  {{ isSubmitting ? $t('common.submitting') : $t('common.submit') }}
</button>

<!-- ② 핸들러: 4-레이어 구조 -->
const isSubmitting = ref(false)

async function onSubmit() {
  if (isSubmitting.value) return          // Layer 1: 중복 실행 방지 (최상단 필수)
  isSubmitting.value = true               // Layer 2: 즉시 잠금
  try {
    await store.doSomething()             // Layer 3: 실제 작업
  } catch (e) {
    const code = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    errorMsg.value = t(`errors.${code}`, t('errors.ERR_UNKNOWN'))
  } finally {
    isSubmitting.value = false            // Layer 4: 항상 해제 (성공·실패 무관)
  }
}
```

**규칙:**
- `if (isSubmitting.value) return` — 핸들러 **첫 줄**에 반드시 존재. `disabled`만으로는 Enter 키·프로그래매틱 호출을 막을 수 없다.
- `isSubmitting.value = true` — try 블록 **바깥 위**에 위치. try 안에 두면 예외 발생 시 잠금이 풀리지 않는다.
- `finally { isSubmitting.value = false }` — 성공·실패·예외 모든 경로에서 해제.
- 상태 변수명: `isSubmitting`, `isDeleting`, `isSaving` 등 `is` + 동명사 형태.
- 폼 입력 필드도 `:disabled="isSubmitting"` 적용 — 제출 중 수정 방지.
- **프론트엔드 자체 검증 오류** (비밀번호 불일치 등)는 `t('ns.key')` 직접 사용 가능. 백엔드 응답 오류만 `t('errors.ERR_CODE')` 패턴 사용.

### API
- 모든 API 경로: `/api/` 접두어.
- 인증: `cc_session` 쿠키. fetch 시 `credentials: 'include'` 필수.
- 서버에서 쿠키 검증: `auth_sessions` 테이블에서 토큰 조회 + `expires_at > datetime('now')` 확인.
- **오류 응답 형식**: `{ "error": "ERR_CODE" }`. 오류 코드는 `SCREAMING_SNAKE_CASE`.
- **백엔드 오류 메시지 하드코딩 금지**: 백엔드는 오류 코드만 반환. 프론트엔드가 `$t('errors.ERR_CODE')` 형태로 번역. `ApiError::BadRequest("한국어 문자열")` 사용 금지.
- **지원 언어 목록은 프론트엔드가 관리**: 백엔드는 특정 언어 코드 목록을 하드코딩하지 않는다. 형식 검증만 수행하고, 알 수 없는 값은 기본값(`en`)으로 대체한다.

---

## PROHIBITED

- **Silent failures**: 모든 에러 명시적 처리, 사용자 화면에 표시.
- **컴포넌트에서 직접 API 호출**: store를 통하지 않은 호출 금지.
- **프론트엔드 비즈니스 로직**: 채점, 권한 판단, 점수 계산은 서버에서.
- **하드코딩된 URL**: 모든 API 호출은 `api/client.ts`의 `api` 객체를 통한다.
- **`any` 타입**: TypeScript에서 `any` 사용 금지.
- **`sqlx::query!` 매크로**: 컴파일 타임 DB 접근 요구하므로 사용 금지.
- **Polling 방식**: setInterval 기반 반복 fetch 사용 금지. 실시간 데이터는 WebSocket(학생) 또는 SSE(교사)로 처리.
- **JWT**: 인증에 JWT 사용 금지. 쿠키 세션 방식만 사용.
- **미구현 stub 방치**: placeholder는 최소한 로딩/에러 상태 UI를 갖춰야 한다.
- **CSS 하드코딩 색상**: 반드시 CSS 변수 사용. `#2563eb` 직접 사용 금지.
- **`text-sm` 이하 폰트 클래스**: `text-sm`, `text-xs` 등 `text-base`(16px)보다 작은 Tailwind 폰트 클래스 사용 금지.
- **외부 CDN 폰트·아이콘**: `fonts.googleapis.com` 등 외부 CDN 링크 사용 금지. 모든 폰트·아이콘은 npm 패키지로 번들에 포함.
- **UI 텍스트 한국어 하드코딩**: 모든 UI 텍스트는 `$t('key')` 형태로만 출력. 컴포넌트 템플릿에 한국어 문자열 직접 삽입 금지.
- **백엔드 오류 메시지 한국어 하드코딩**: `ApiError::BadRequest("한국어...")` 금지. 반드시 `ApiError::BadRequest("ERR_CODE")` 형태로 오류 코드를 반환하고, 프론트엔드 `locales/{lang}/errors.json`에서 번역.
- **색만으로 상태 전달**: 상태 표시 시 색 + 텍스트를 항상 함께 사용. 색각 이상 사용자를 고려.
- **파일 임포트 시 열 인덱스 사용**: CSV/XLSX 파싱 시 열 인덱스(0, 1, 2...) 접근 금지. 반드시 열 이름으로 매핑하며, 매핑 사전을 통해 동의어 처리.
- **모달 외부 클릭 닫기**: `@click.self`로 모달 바깥 클릭 시 닫히도록 구현 금지. 모달은 ESC 키로만 닫힌다.
- **버튼 가드 누락**: 비동기 액션 핸들러 첫 줄에 `if (isSubmitting.value) return` 없이 구현 금지. `:disabled` 바인딩만으로는 중복 실행을 완전히 막을 수 없다.
- **`style=""` 속성에 CSS 변수 외 값 작성**: `style="padding: 0"`, `style="border-radius: 8px"`, `style="display: flex"` 등 Tailwind 클래스로 표현 가능한 값을 inline으로 작성 금지. CSS 변수(`var(--xxx)`)를 값으로 쓰는 경우만 `style=""` 허용.
- **전역 폼 스타일을 `@layer base` 밖에 선언**: `base.css`에 `button {}`, `input {}` 등을 unlayered로 작성하면 Tailwind 유틸리티 클래스 전체가 무력화된다. 반드시 `@layer base { }` 안에 선언할 것.

---

## PROJECT STRUCTURE

```
docs/
  design-decisions.md     — 설계 결정 사항 (0~12장), 구현 시 항상 참고
  mockups/                — HTML 목업 파일 (구현 참고용)

src-tauri/
  Cargo.toml
  tauri.conf.json
  migrations/
    001_core.sql          — app_configs, subjects, teachers, students, auth_sessions
    002_classes.sql       — classes, class_students, languages, class_allowed_languages
    003_lessons.sql       — lessons, class_lessons
    004_problems.sql      — problem_types, problems, problem_*, test_cases
    005_assessments.sql   — assessments, class_assessments, assessment_problems
    006_sessions.sql      — assessment_sessions, assessment_session_targets
    007_submissions.sql   — (논의 예정)
    008_attendance.sql    — (논의 예정)
  src/
    main.rs               — 진입점 (run() 호출)
    lib.rs                — Tauri 빌더, 트레이, Axum 서버 spawn
    db.rs                 — SQLitePool 초기화, WAL, 마이그레이션 실행
    error.rs              — ApiError (IntoResponse impl)
    server/
      mod.rs              — Axum 라우터 조립, 서버 기동
      state.rs            — AppState { db: SqlitePool }
      routes/
        mod.rs
        setup.rs          — GET /api/setup/status, POST /api/setup/complete
        auth.rs           — POST /api/auth/login/teacher|student, /logout, GET /me
        classes.rs        — GET/POST /api/classes, PUT/DELETE /api/classes/:id
        admin.rs          — /api/admin/teachers, /api/admin/subjects (admin 전용)

frontend/
  package.json
  vite.config.ts          — /api 프록시 → localhost:8080
  src/
    main.ts
    App.vue
    assets/
      main.css            — @import 진입점 (4줄: tailwindcss, pretendard, tokens, base)
      tokens.css          — CSS 디자인 토큰 (색상, 간격, 반지름 변수 — 라이트/다크)
      base.css            — 리셋, body, @layer base 폼 스타일, .spin 애니메이션
    api/client.ts         — fetch 기반 API 헬퍼 (모든 엔드포인트 집중 관리)
    router/index.ts       — 라우트 + role 기반 네비게이션 가드
    stores/
      auth.ts             — 교사/학생 세션, role, 학교 이름
      class.ts            — 수업 목록, 과목 목록, 수업 CRUD
      admin.ts            — 교사 계정 관리, 과목 관리 (admin 전용)
    views/
      SetupView.vue       — 초기 설정 (0장)
      LoginView.vue       — 교사/학생 탭 로그인 (role→라우팅)
      TeacherHomeView.vue — 교사 홈 (수업 카드 그리드, 수업 CRUD 모달)
      AdminView.vue       — 관리자 홈 (교사 계정·과목 관리)
      StudentHomeView.vue — 학생 홈 (진행 중 세션 진입 또는 대기 화면)
    components/           — 공용 컴포넌트 (LanguageSelector.vue 등)
    locales/
      ko/ en/             — setup, auth, common, errors, classes, admin, student
```

### 라우트 구조
| 경로 | 뷰 | 가드 |
|------|-----|------|
| `/setup` | SetupView | needs_setup |
| `/login` | LoginView | 비로그인 |
| `/teacher` | TeacherHomeView | requiresAuth: 'teacher' (또는 admin) |
| `/admin` | AdminView | requiresAuth: 'admin' |
| `/student` | StudentHomeView | requiresAuth: 'student' |
| `/student/change-password` | (인라인/모달) | requiresAuth: 'student' |

---

## DB SCHEMA (Key Rules)

- WAL 모드 + `foreign_keys=ON` 항상 적용 (`db.rs` 초기화 시 PRAGMA 실행)
- SQLite 단일 writer 큐: `SqlitePoolOptions::max_connections(1)`
- 모든 PK: `id INTEGER PRIMARY KEY`
- 타임스탬프: `TEXT NOT NULL DEFAULT (datetime('now'))` (SQLite ISO 8601 UTC)
- `type_config TEXT NOT NULL DEFAULT '{}'`: 문항 유형별 설정 JSON
- `auth_tokens`: 교사 세션 토큰 (expires_at 12시간)
- `sessions`: 수행평가 세션 전용 — `auth_tokens`와 별개, 이름 혼동 주의
- **`divisions` 테이블 폐지** — `classes`로 전면 대체. 코드에서 `division` 참조 금지.
- **`lesson_releases`, `assessment_divisions` 테이블 폐지** — `lessons.is_released`, `assessments.class_id`로 단순화.

### 핵심 테이블 변경 이력 (구 → 신)
| 구 | 신 | 비고 |
|----|----|----|
| `divisions` | `classes` | `subject`, `grade`, `class_no` 필드 추가 |
| `teacher_divisions` | `teacher_classes` | FK `class_id` |
| `students.division_id` | `students.class_id` | |
| `lesson_releases` | (폐지) | `lessons.is_released` + `lessons.class_id`로 대체 |
| `assessment_divisions` | (폐지) | `assessments.class_id`로 대체 |
| `sessions.division_id` | `sessions.class_id` | |

### 마이그레이션 규칙 (릴리즈 전)
- 스키마는 도메인별로 분리된 파일로 관리한다:
  - `001_core.sql` — app_configs, subjects, teachers, students, auth_sessions
  - `002_classes.sql` — classes, class_students, languages, class_allowed_languages
  - `003_lessons.sql` — lessons, class_lessons
  - `004_problems.sql` — problem_types, problems, problem_*, test_cases
  - `005_assessments.sql` — assessments, class_assessments, assessment_problems
  - `006_sessions.sql` — assessment_sessions, assessment_session_targets
  - `007_submissions.sql` — (논의 예정)
  - `008_attendance.sql` — (논의 예정)
- **릴리즈 전**: 해당 도메인 파일 직접 수정. 새 번호 파일 추가 금지.
- **릴리즈 후**: `009_xxx.sql` 형태로 증분 마이그레이션 추가.
- 변경 후 테스트 DB 파일을 삭제하면 앱 재기동 시 최신 스키마로 재생성된다.

### 도메인 모델 (핵심 계층)
```
classes (수업: 과목+분반 조합, 예: "정보과학 1반")
  └─ teacher_id (담당 교사)
  └─ students (학생 명단)
  └─ lessons (차시)
  └─ assessments (수행평가)
       └─ sessions: assessment_id + class_id
            └─ submissions (제출)
            └─ submission_drafts (임시저장)
problems (문항) — 전역 공유
attendance_records (출석 스냅샷)
```

### 파일 임포트 열 이름 매핑 사전
CSV/XLSX 임포트 시 아래 매핑 사전을 사용. 열 인덱스 접근 금지.
```
student_number: ["번호", "학번", "No", "number", "student_no"]
name:           ["이름", "성명", "학생명", "name", "student_name"]
grade:          ["학년", "grade", "year"]
class_no:       ["반", "학반", "class", "division"]
```

---

## GIT / COMMIT RULES

- **GPG 서명 필수**: 모든 커밋에 `-S` 플래그 사용. `git commit -S -m "..."`
- **Co-Authored-By / Co-Worked 문구 삽입 금지**: 커밋 메시지에 Claude 관련 문구 일절 포함하지 않는다.
- 커밋 메시지: 한국어 또는 영어, 간결하게 작성.

---

## BUILD / DEV

- **Rust 빌드**: `cargo check --manifest-path src-tauri/Cargo.toml` (사용자가 직접 실행)
- **프론트엔드 빌드**: `cd frontend && npm run build`
- **개발 서버**: 사용자가 직접 `cargo tauri dev` 실행. Claude는 서버를 직접 기동하지 않는다.
- 빌드 확인이 필요한 단계에서는 사용자에게 빌드 요청 후 결과를 보고받는다.
