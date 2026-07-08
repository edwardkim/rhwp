# 구현계획서 — task_m100_2050

- **이슈**: #2050 [vscode] 좌측 사이드바 (썸네일 + 목차 + 북마크)
- **브랜치**: `local/task2050`
- 수행계획서: `task_m100_2050.md`
- **PR 전략**: 단일 PR (Phase A~C 한 브랜치, pkg 재빌드 포함)
- **목차 모드**: `auto` 기본

## 단계별 구현 (5단계)

### Stage 1 — Rust: `getStructure` WASM 바인딩

1. `src/document_core/queries/structure.rs`(또는 동 모듈)에 코어 메서드 추가:
   `pub fn get_structure_native(&self, mode: &str) -> Result<String, HwpError>`
   - `StructureMode::parse(mode)` (실패 시 `auto` 폴백) → `build_structure(&self.document, mode)` → `serde_json` 직렬화.
   - 기존 `get_bookmarks_native` 패턴 준수.
2. `src/wasm_api.rs`에 바인딩 추가:
   ```rust
   #[wasm_bindgen(js_name = getStructure)]
   pub fn get_structure(&self, mode: &str) -> Result<String, JsValue> {
       self.core.get_structure_native(mode).map_err(|e| e.into())
   }
   ```
3. 검증: `cargo build`, `cargo test` (네이티브, 로컬). 파서/렌더/레이아웃 무변경 확인.

산출: 소스 커밋 + `_stage1.md`.

### Stage 2 — WASM 재빌드 + pkg 갱신

1. Docker WASM 빌드: `docker compose --env-file .env.docker run --rm wasm` → `pkg/` 갱신.
2. `pkg/rhwp.d.ts`에 `getStructure(mode: string): string` 노출 확인.
3. 검증: 빌드 성공, 타입 정의 존재.

산출: pkg 갱신 커밋 + `_stage2.md`.

### Stage 3 — vscode: 사이드바 레이아웃 + 썸네일

1. `hwp-editor-provider.ts` HTML: 최상위를 `#app-shell`(flex row)로 감싸 `#nav-sidebar`(좌) + 기존 뷰(우) 배치. `#scroll-container` **내부 구조·좌표계 불변**.
   - 사이드바: 탭 헤더(썸네일/목차/북마크) + 콘텐츠 영역 + 접기 토글.
   - CSS 접두어 `nav-` 신설(기존 규약 tb-/sb-/stb- 계열과 분리).
2. `viewer.ts`: 썸네일 목록 생성.
   - 각 페이지 축소 캔버스(고정 폭, 비율 유지), **IntersectionObserver 지연 렌더**(`renderPageToCanvas(i, canvas, thumbScale)`).
   - 클릭 → 해당 `pageInfos[i].element`로 스크롤. 현재 페이지 하이라이트(기존 `updateCurrentPage` 연동).
3. 검증: webpack 빌드, 썸네일 표시·클릭 이동.

산출: 소스 커밋 + `_stage3.md`.

### Stage 4 — vscode: 목차 + 북마크 패널 + 네비게이션

1. 목차: 로드 후 `hwpDoc.getStructure("auto")` → JSON 트리 렌더(중첩 들여쓰기). 노드 클릭 → `getPageOfPosition(section, para)` → 페이지 스크롤. 빈 트리 시 "목차 없음" 안내.
2. 북마크: `hwpDoc.getBookmarks()` → 목록 렌더. 클릭 → `getPageOfPosition` → 이동. 빈 목록 시 안내.
3. 탭 전환 로직(썸네일/목차/북마크) 연결.
4. **하단 쪽번호 이동** (추가 요청): 상태 표시줄 `#stb-page` 클릭 → 쪽번호 입력(작은 input/프롬프트) → 해당 페이지로 스크롤. 기존 페이지 스크롤 로직 재사용.
5. 검증: webpack 빌드, 목차·북마크·쪽번호 클릭 이동.

> 참고: 목차 클릭 이동은 본 Stage에 이미 포함. "2쪽 보기"는 별도 이슈 #2051로 분리.

산출: 소스 커밋 + `_stage4.md`.

### Stage 5 — 최종 검증 + 보고 + PR

1. 전체 재빌드(`npm run compile`), 필요 시 `npm run package` vsix.
2. 실기기 시각 판정(작업지시자): 썸네일/목차/북마크 이동, 접기 토글, 기존 스크롤·줌 회귀 없음.
3. 최종 보고서 `report/task_m100_2050_report.md`.
4. 문서·소스 커밋 → 단일 PR (base `devel`, head `planet6897:pr-task2050`).

산출: `_report.md` + PR.

## 비적용 범위

- 파서/렌더러/레이아웃/문서코어 로직 변경 없음 (읽기 전용 질의 노출만).
- HWP3 전용 분기 없음.
- `cargo fmt --all` 미실행 (기능 브랜치, 신규/수정 파일만 정리).

## 검증 기준 (종합)

- `cargo build`/`cargo test` 통과, Docker WASM 재빌드 성공.
- webpack 빌드 통과.
- 썸네일·목차·북마크 클릭 이동 정상, 사이드바 접기 동작, 기존 스크롤/줌 회귀 없음.
