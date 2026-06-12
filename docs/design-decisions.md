# CodeClan LMS - 정보 교과 LMS + 온라인 저지 설계 결정 사항 (최종)

마지막 업데이트: 2026-06-12

프로젝트명: **CodeClan LMS** (레포지토리명 예정: `codeclan-lms`)

## 0. 최초 부팅 / 초기 설정 플로우

- 앱 최초 실행 시 (DB에 교사 계정이 0개) → 강제로 초기 설정 화면 진입 (다른 화면 접근 불가)
- 초기 설정 화면 입력 항목: **학교 이름**(설정 테이블에 단일 행으로 저장), **관리자 계정** (이름/아이디/비밀번호)
- 학교 이름은 이후 로그인 화면 환영 문구("환영합니다, OO고등학교 LMS입니다")에 사용
- 생성되는 계정은 자동으로 권한="관리자"
- 설정 완료 후 일반 로그인 화면으로 리다이렉트. 이후 재실행 시 초기 설정 화면은 표시되지 않음 (DB에 계정 존재 여부로 판단)
- 비밀번호는 해시(argon2) 저장. 세션 인증 방식은 백엔드 설계 단계에서 결정 (쿠키 기반 세션 토큰 예정)

## 1. 아키텍처 (확정, 변경 없음)

- Tauri 2.0 래퍼 (Windows), 시스템 트레이 상주 + 닫기 시 숨김
- Axum 서버, 0.0.0.0:8080 바인딩 (학내망)
- Vue 3 + Vite + TypeScript 프론트엔드, Rust 바이너리에 임베드
- SQLite WAL, 단일 writer 큐
- DMOJ wbox 채점 엔진, 포터블 Python/GCC/Java 런타임
- 교사: 웹 + Tauri 트레이 앱 모두 접근 가능
- 학생: 학번 + 생년월일 로그인

## 2. 문제 은행 / 문항 유형

문제 은행은 전역 단일 마스터. 평소수업(차시, session_id=NULL)과 수행평가(session_id=설정) 모두에서 재사용.

제출 버전 관리: `submission_no` 증가 + `is_latest=1` 플래그.

### 5가지 문항 유형

1. **①실행결과맞히기** - 코드 표시, 학생이 실행 결과를 텍스트로 입력
2. **②코드작성형** - 온라인 저지. 학생이 코드 작성/제출, 테스트케이스로 자동채점
3. **③과제/보고서형** (구 "포트폴리오형") - 구글 클래스룸 "학생마다 사본 제출" 패턴.
   - 교사가 docx 템플릿 업로드 → 학생별 사본 생성 (`uploads/{student_submission_uuid}.docx`)
   - 학생이 브라우저 내에서 직접 편집 (Vue 3용 오픈소스 docx 에디터, 예: eigenpal/docx-editor - Apache 2.0, OOXML round-trip)
   - `type_config.has_code: bool`이면 2단 보기(코드+문서 동시 편집/제출) 지원
   - 자동채점 대상 아님, 교사 수기 채점
   - (미래 차시 기능 아이디어) 짝 프로그래밍 - 복수 학생 동시 편집은 추후 별도 검토. 필요 시 SuperDoc(Yjs 협업) 등으로 교체 검토
4. **④빈칸채우기** - Monaco 마커 영역으로 동떨어진 여러 구간을 개별 편집 가능하게 지정
5. **⑤구조검사** - 별도 유형 아님. ②/④에 옵션으로 붙는 "사전 게이트" 플래그 (구조/키워드 검사)

### 확장성 패턴

- 공통 메타데이터 + `type_config` JSON 컬럼 → 새 유형 추가 시 마이그레이션 불필요
- 프론트엔드 "유형 레지스트리": 유형 → (작성/풀이/결과) 컴포넌트 매핑
- 컴포넌트 구조: 합성 가능한 것은 합성, 유형별로 다른 부분은 독립 뷰로 분리 (마법사 강제 X)

## 3. 차시(Lesson) / 분반(Division) / 수행평가(Assessment) 구조

- **차시**: 교과 단위 공통 (여러 분반이 공유), 차시 목록/배정문항은 분반별로 커스터마이징 가능, 공개는 분반별 수동 토글 (LessonRelease 매핑)
- **분반**: 예) "프로그래밍 308" - 자체 명부(학년/반 범위로 일괄 등록 + 수동 조정), 자체 교사, 자체 차시 목록. 단, 동일 교과의 분반들은 같은 전역 Problem Bank + 같은 Assessment를 공유
- **Assessment(수행평가)**: 전역 엔티티. 여러 분반이 link로 연결 (복사 아님). 수정 시 연결된 모든 분반에 즉시 반영. **편집 잠금**: 연결된 분반 중 하나라도 RUNNING 세션이 있으면 제목/설명/배정문항/배점 편집 전체 비활성화
- 분반 연결 해제 시: 해당 분반 학생은 더 이상 조회 불가하나, 기존 세션/제출 기록은 보존. RUNNING인 분반은 연결 해제 불가
- 공개 여부(차시)·연결(Assessment) 모두 분반 단위로 결정

## 4. 세션(Session) 라이프사이클

`status` enum: **CREATED ⇄ LOBBY → RUNNING → CLOSED**

- CREATED ⇄ LOBBY: 양방향 (취소/재구성 자유, 학생 데이터 없음)
- LOBBY → RUNNING → CLOSED: 단방향. RUNNING 클릭이 비가역 경계
- `is_paused: bool` + `paused_at`: RUNNING 상태의 직교 플래그. 일시중단/재개 시 `end_at` 자동 연장 (크래시 복구 타이머 로직 재사용)
- `is_result_released: bool`: 분반/세션 범위, 양방향 토글, 양방향 모두 단순 확인(예/아니오) 다이얼로그만, 사유 입력/저장 없음
- 추가시험(makeup exam): 같은 Assessment를 참조하는 새 Session 행, 특정 학생 1인 범위, 독립 라이프사이클/독립 제출 누적

### 세션 생성 - 대상 선택

라디오 2개 (기본값: 분반 전체):
1. **분반 전체** - 명부 등록 학생 전체
2. **개별 학생 선택** - 검색 가능한 체크리스트, 재시험/추가시험(makeup exam) 용도. "직전 세션 미응시" 등 컨텍스트 힌트 표시

세션 생성 직후 항상 **CREATED**로 생성 → 교사가 LOBBY로 전환. 결과 공개 토글은 세션 생성 화면에 노출하지 않고, CLOSED 이후 별도 화면에서 `is_result_released` 토글.

### 지각 입장 처리

- 세션 공유 `end_at`. 늦은 학생은 RUNNING에 자동 합류, 남은 시간 단축
- 교사 출결 위젯에 "지각 입장 HH:MM" 배지 + 감사 로그
- CLOSED 이후 로그인 시 "이미 종료된 평가입니다" 메시지로 차단

### 실시간 접속 현황(출결) 위젯

- 하트비트 기반 초록/회색 점. 차시 운영 화면 + 평가 LOBBY/RUNNING 화면에서 공통 재사용

## 5. 학생 UI 구조

- 2단계 네비게이션: 교과 선택(카드 그리드) → 수업(차시) / 수행평가 탭 (절대 혼합 안 함, URL도 `/student/lessons` vs `/student/assessments` 분리)
- 분반에 RUNNING 세션이 있으면 강제 "시험 모드" 전체화면 진입, 차시 네비게이션 전체 숨김
- 수행평가 탭(평상시): 예정 안내 + 지난 결과 목록 (결과 미공개/공개 상태 구분)

## 6. 문항 등록 (교사)

- 유형별 독립 Vue 뷰 컴포넌트 + 공유 서브 위젯 (TestCaseEditor: ZIP 업로드/드래그드롭/wbox 자동생성, LanguageSelector, AdvancedLimitsPanel, StructureCheckPanel, MonacoMarkerEditor)
- 모든 단계에서 "임시저장" 지원
- "문제 복사": 유형+설정 복제, 제목/설명/테스트케이스는 초기화

## 7. 교사 네비게이션 구조 (최종)

대시보드를 진입점으로, "수업 준비"와 "수업 운영"을 명확히 분리. 사이드바는 3개 그룹으로 재구성:

- **(그룹 없음) 대시보드**: 담당 분반별 "수업 운영 - 지금 들어가기" 카드(상단, 강조) + "수업 준비" 카드 그리드(하단)
- **수업 그룹**: 시험 세션 운영 (RUNNING 세션 매트릭스/감사 타임라인)
- **준비 그룹**: 문제 은행 / 차시 관리 / 수행평가 관리 (모두 교과 단위 전역 공유 리소스 또는 분반별 배정·공개 설정)
- **관리 그룹**: 학생·반 관리(명부/출결 기록) / 감사 로그 / 백업·시스템 / 교사 계정 설정

대시보드 → "차시 운영 시작" 진입 시 신규 화면(차시 운영 라이브 화면)으로 이동:
- 차시 설명/공개 토글 ("지금 공개하기" - 클릭 즉시 학생 화면 노출)
- 배정 문항을 순서대로 진행(이전/다음 네비게이션), 진행중 문항 강조 + "학생화면 미리보기"
- 우측 실시간 접속 현황(출결) 위젯
- 하단 "이어서 수행평가 진행" 카드 - 연결된 Assessment가 있으면 세션 생성/시작으로 즉시 연결

> **반영 상태**: 위 네비게이션 구조는 합의된 최종안. 현재 목업(`teacher_dashboard_mode_split.html`, `lesson_live_operation_screen.html` 등)의 사이드바는 아직 단일 평면 목록(대시보드/문제은행/수행평가/시험세션운영/학생·반관리/감사로그/백업시스템)으로, 3그룹 구분이 시각적으로 반영되지 않은 상태. 차기 리비전에서 사이드바 컴포넌트에 그룹 헤더(준비/수업/관리)를 추가하는 일괄 수정 필요. 기능/화면 설계 자체는 변경 없음.

## 8. 분반 연결 / 세션 생성 UI 세부

- **전역 목록 + 상세 2단 레이아웃**: 차시 관리 화면과 동일한 좌(목록)-우(상세) 패턴. 좌측 목록에 "연결된 분반 수" 배지, RUNNING 세션이 하나라도 있으면 "운영중" 빨간 배지
- **분반 연결 카드**: 분반별로 현재 세션 상태에 따라 액션 버튼 분기
  - 세션 없음 → "세션 생성"
  - RUNNING → "세션 운영 화면으로"
  - CLOSED → "세션 기록" + "추가시험 생성"
- 분반 연결 모달에 "복사가 아닌 참조" 안내 문구 명시

## 9. 완료된 목업 (mockups/)

| 파일 | 내용 |
|---|---|
| `teacher_admin_gui_layout.html` | 교사 admin GUI 레이아웃 - 사이드바 + 실시간 제출 매트릭스 + 감사 타임라인 + 시스템 상태 |
| `teacher_dashboard_mode_split.html` | 교사 대시보드 (최종) - 수업 운영/수업 준비 분리 |
| `teacher_dashboard_landing.html` | 교사 대시보드 1차 시안 (구버전, 참고용 보존) |
| `lesson_live_operation_screen.html` | 차시 운영(라이브) 화면 - 공개 토글, 문항 진행, 출결 위젯, 수행평가 연계 |
| `lesson_management_screen.html` | 교사용 차시 관리 화면 - 차시 목록/배정문항/반별 공개 상태 |
| `problem_authoring_wizard.html` | 문항 등록 워크플로우 다이어그램 |
| `problem_wizard_step3_code_judge.html`, `problem_wizard_step3_refined.html` | 문항 등록 마법사 3단계 - ②코드작성형 |
| `problem_wizard_type1_type4_quicklook.html` | 문항 등록 마법사 - ①/④ 퀵룩 |
| `assessment_management_screen.html` | 수행평가 관리 - 전역 Assessment 목록/상세, 배정 문항, 연결된 분반 현황 |
| `assessment_division_link_modal.html` | 분반 연결 관리 모달 |
| `assessment_session_create.html` | 수행평가 세션 생성 화면 - 분반전체/개별학생 라디오 |
| `student_main_subject_selection.html` | 학생 메인 화면 - 교과 카드 그리드 |
| `student_lesson_vs_assessment_separation.html` | 학생 차시/수행평가 탭 분리 비교 (일반 vs 강제 시험모드) |
| `student_assessment_tab_normal_state.html` | 학생 수행평가 탭 - 예정 안내 + 지난 결과 |
| `student_solving_type1_type4.html`, `student_solving_type2_type3.html`, `student-solving-screens.html` | 학생 풀이 화면 - ①②③④ 전체 유형 |
| `session_lobby_screen.html` | 세션 LOBBY 화면 - 출결 위젯 + 시작(RUNNING, 비가역)/취소(CREATED 복귀) |
| `teacher_account_management.html` | 계정 설정 - 교사 계정 목록, 권한 레벨(관리자/일반 교사), 담당 분반 |
| `welcome_initial_setup_screen.html` | 최초 부팅 초기 설정 - 학교 이름 + 관리자 계정 생성 |

## 10. 사이드바 3그룹화 반영 완료

`teacher_admin_gui_layout.html`, `teacher_dashboard_mode_split.html`의 사이드바를 7장 최종안대로 재구성:
- (그룹 없음) 대시보드
- **수업**: 시험 세션 운영
- **준비**: 문제 은행 / 차시 관리 / 수행평가
- **관리**: 학생/반 관리 / 감사 로그 / 백업·시스템 / 계정 설정

권한 레벨은 2단계로 정의: **관리자**(전 분반/교과 공유 리소스 편집, 계정/백업/감사로그 전체), **일반 교사**(담당 분반 운영, 공유 리소스 조회·사용).

## 11. DB 스키마 초안 (ERD)

SQLite 기준. 모든 PK는 `id INTEGER PRIMARY KEY`, 타임스탬프는 `created_at`(기본 now) 형식. 아래는 구현 단계에서 세부 컬럼/제약을 조정할 수 있는 초안.

### 11.1 조직/계정

| 테이블 | 주요 컬럼 | 설명 |
|---|---|---|
| `settings` | `school_name`, `updated_at` | 단일 행. 0장 초기 설정에서 입력 |
| `teachers` | `username` UNIQUE, `password_hash`, `name`, `role`(admin\|teacher), `created_at` | 0장에서 최초 admin 생성 |
| `auth_tokens` | `teacher_id`, `token` UNIQUE, `created_at`, `expires_at` | 교사 쿠키 기반 세션 토큰. `sessions` 테이블은 수행평가 세션 전용이므로 분리 |
| `divisions` | `name`, `created_at` | 분반 (예: "3학년 2반") |
| `teacher_divisions` | `teacher_id`, `division_id` | 일반교사-분반 M:N (admin은 전체 접근이라 행 불필요) |
| `students` | `division_id`, `student_number`(학번) UNIQUE, `name`, `birth_date` | 로그인 = 학번+생년월일 |

### 11.2 문제 은행 / 차시 / 수행평가 (전역 공유 리소스)

| 테이블 | 주요 컬럼 | 설명 |
|---|---|---|
| `problems` | `type`(1~5), `title`, `description`, `type_config`(JSON), `is_structure_check`, `created_at` | 전역 단일 마스터. `type_config`에 유형별 설정(테스트케이스, 빈칸마커, has_code 등) |
| `lessons` | `title`, `description`, `order_no`, `created_at` | 교과 단위 공통(분반 공유) |
| `lesson_problems` | `lesson_id`, `problem_id`, `order_no` | 차시별 배정 문항 (session_id=NULL로 problems 재사용) |
| `lesson_releases` | `lesson_id`, `division_id`, `is_released`, `released_at` | 분반별 공개 토글 |
| `assessments` | `title`, `description`, `created_at` | 전역 엔티티. 편집 잠금은 연결된 분반의 세션 상태로 판단(별도 컬럼 없음) |
| `assessment_problems` | `assessment_id`, `problem_id`, `order_no`, `score` | 배정 문항 + 배점 |
| `assessment_divisions` | `assessment_id`, `division_id` | 분반 연결(참조, 복사 아님). 해제 시 행 삭제하되 세션/제출 기록은 보존 |

### 11.3 세션 / 제출 / 출결

| 테이블 | 주요 컬럼 | 설명 |
|---|---|---|
| `sessions` | `assessment_id`, `division_id`, `status`(CREATED\|LOBBY\|RUNNING\|CLOSED), `target_type`(ALL\|INDIVIDUAL), `time_limit_min`, `start_at`, `end_at`, `is_paused`, `paused_at`, `is_result_released`, `parent_session_id`(추가시험용, nullable), `created_at` | 추가시험 = 같은 `assessment_id` 참조하는 새 행, `target_type=INDIVIDUAL` |
| `session_targets` | `session_id`, `student_id` | `target_type=INDIVIDUAL`일 때만 사용 (분반전체면 비어있음 = 명부 전체 대상) |
| `submissions` | `problem_id`, `student_id`, `session_id`(nullable, 평소수업=NULL), `submission_no`, `is_latest`, `language`, `content`, `verdict`, `score`, `judged_at`, `created_at` | wbox 채점 결과 반영. 재제출 시 `submission_no`+1, 기존 `is_latest=0` |
| `attendance_heartbeats` | `student_id`, `context_type`(lesson\|session), `context_id`, `joined_at`, `last_seen_at`, `is_late` | 실시간 접속 위젯의 데이터 소스 |
| `audit_logs` | `actor_teacher_id`(nullable=시스템), `action_type`, `target_type`, `target_id`, `detail`, `created_at` | 점수 수동수정, 세션 시작/종료, 결과공개 토글, 비밀번호 재설정 등 기록 |

### 11.4 관계 요약

- `divisions` 1—N `students`, `teachers` N—M `divisions`
- `lessons` N—M `problems` (via `lesson_problems`), `lessons` N—M `divisions` (via `lesson_releases`, 공개여부 포함)
- `assessments` N—M `problems` (via `assessment_problems`), `assessments` N—M `divisions` (via `assessment_divisions`)
- `sessions` N—1 `assessments`, N—1 `divisions`; `sessions` 1—N `submissions`(session_id 경유)
- `submissions` N—1 `problems`, N—1 `students`; 평소수업 제출은 `session_id=NULL`로 동일 테이블 재사용

### 11.5 다음 단계

- 위 초안을 기반으로 Claude Code 환경에서 SQLite 마이그레이션 파일 작성, sqlx 타입/쿼리, Axum 라우트 설계 진행
- `type_config`(JSON) 세부 스키마(유형별 필드)는 문항 등록 마법사 구현 시점에 구체화

## 12. 향후 작업

- 구현 시작: Welcome/초기설정 화면(0장) → 관리자 계정 생성 → DB 스키마 설계 → 백엔드(Axum)/프론트(Vue) 순서로 단계별 연결
- DB 스키마 설계, Rust/Tauri 백엔드 구조 설계, wbox 채점 엔진 통합은 Claude Code 환경에서 진행
- 나머지 화면 목업은 구현 중 계속 수정될 예정
