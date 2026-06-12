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
- 향후 추가: `stores/student.ts`, `stores/lesson.ts`, `stores/assessment.ts`, `stores/problem.ts`

---

## PRINCIPLES

- **Store = Single Source of Truth**: 서버 응답을 컴포넌트가 직접 보유하지 않는다.
- **Frontend = UI + 상태**: 비즈니스 규칙, 채점, 권한 판단은 모두 서버에 있다.
- **중복 로직 금지**: 서버가 처리하는 것을 프론트엔드에서 재구현하지 않는다.
- **단일 바이너리 원칙**: Tauri + Axum이 하나의 실행 파일 안에서 동작한다.

---

## UI / DESIGN RULES

- **CSS 변수(design token) 기반**: 색상, 간격, 반지름은 `frontend/src/assets/main.css`의 `:root` 변수를 참조한다. 인라인 하드코딩 금지.
- Vue 컴포넌트 스타일은 `<style scoped>` 안에 작성한다.
- **아이콘: `@tabler/icons-vue` 패키지** 사용. 패키지가 없는 경우 SVG inline 대체.
- 모든 교사 화면은 **사이드바 + 메인 콘텐츠** 레이아웃.
- 로딩 상태와 에러 상태를 반드시 UI에 표시한다 (스피너, 에러 배너).
- 교사 화면: 최소 1024px 기준 / 학생 화면: 768px 이상 기준.
- 구현 시 **`docs/mockups/*.html`** 을 참고한다. 레이아웃·색상 구조를 그대로 따른다.

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

### API
- 모든 API 경로: `/api/` 접두어.
- 인증: `cc_session` 쿠키. fetch 시 `credentials: 'include'` 필수.
- 서버에서 쿠키 검증: `auth_tokens` 테이블에서 토큰 조회 + `expires_at > datetime('now')` 확인.

---

## PROHIBITED

- **Silent failures**: 모든 에러 명시적 처리, 사용자 화면에 표시.
- **컴포넌트에서 직접 API 호출**: store를 통하지 않은 호출 금지.
- **프론트엔드 비즈니스 로직**: 채점, 권한 판단, 점수 계산은 서버에서.
- **하드코딩된 URL**: 모든 API 호출은 `api/client.ts`의 `api` 객체를 통한다.
- **`any` 타입**: TypeScript에서 `any` 사용 금지.
- **`sqlx::query!` 매크로**: 컴파일 타임 DB 접근 요구하므로 사용 금지.
- **JWT**: 인증에 JWT 사용 금지. 쿠키 세션 방식만 사용.
- **미구현 stub 방치**: placeholder는 최소한 로딩/에러 상태 UI를 갖춰야 한다.
- **CSS 하드코딩 색상**: 반드시 CSS 변수 사용. `#2563eb` 직접 사용 금지.

---

## PROJECT STRUCTURE

```
docs/
  design-decisions.md     — 설계 결정 사항 (0~12장), 구현 시 항상 참고
  mockups/                — HTML 목업 파일 (구현 참고용, 21개)

src-tauri/
  Cargo.toml
  tauri.conf.json
  migrations/
    001_initial.sql       — 전체 SQLite 스키마 (마이그레이션)
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
        auth.rs           — POST /login/teacher, /logout, GET /me, /school-name

frontend/
  package.json
  vite.config.ts          — /api 프록시 → localhost:8080
  src/
    main.ts
    App.vue
    assets/main.css       — CSS 변수(디자인 토큰) 정의
    api/client.ts         — fetch 기반 API 헬퍼 (모든 엔드포인트 집중 관리)
    router/index.ts       — 라우트 + 네비게이션 가드 (needs_setup 체크)
    stores/               — Pinia 스토어 (auth.ts, 향후 추가)
    views/
      SetupView.vue       — 초기 설정 (0장)
      LoginView.vue       — 교사/학생 탭 로그인
      DashboardView.vue   — 교사 대시보드
    components/           — 공용 컴포넌트 (향후)
```

---

## DB SCHEMA (Key Rules)

- WAL 모드 + `foreign_keys=ON` 항상 적용 (`db.rs` 초기화 시 PRAGMA 실행)
- SQLite 단일 writer 큐: `SqlitePoolOptions::max_connections(1)`
- 모든 PK: `id INTEGER PRIMARY KEY`
- 타임스탬프: `TEXT NOT NULL DEFAULT (datetime('now'))` (SQLite ISO 8601 UTC)
- `type_config TEXT NOT NULL DEFAULT '{}'`: 문항 유형별 설정 JSON
- `auth_tokens`: 교사 세션 토큰 (expires_at 12시간)
- `sessions`: 수행평가 세션 전용 — `auth_tokens`와 별개, 이름 혼동 주의

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
