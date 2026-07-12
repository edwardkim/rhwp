# Task #2008 Stage 1

## 목표

Studio에서 작은 용량이지만 115쪽 거대 표를 포함한 문서가 로딩 중 멈춘 것처럼 보이는 문제를 개선한다.

## 현재 관찰

- `rhwp-studio/src/main.ts`의 `loadBytes()`는 `wasm.loadDocument()`를 동기 호출한다.
- 동기 호출 중에는 브라우저가 상태창을 다시 그릴 수 없으므로, 상태 메시지를 넣어도 사용자가 못 볼 수 있다.
- `initializeDocument()`의 폰트 로딩만 진행 상태가 있고, 페이지 렌더 준비/도구 모음/편집 상태 초기화 단계는 상태창 단계가 없다.
- `get_cursor_rect_native()`는 초기 커서 rect 계산에서 page tree cache를 쓰지 않는 경로가 있다.
- 표 hover bbox 계산은 page hint 없이 전체 표 bbox를 요청할 수 있어 거대 표 문서의 로드 직후 상호작용 비용을 키울 수 있다.

## Stage 1 범위

- 상태창(`#sb-message`)에 단계 기반 0~100% 로딩 진행률을 표시한다.
- 긴 동기 작업 직전에는 paint 기회를 주어 진행 상태가 실제 화면에 보이도록 한다.
- 초기 커서 rect 계산과 표 hover bbox 계산의 명백한 캐시/힌트 누락을 함께 보정한다.

## 구현 내용

- `rhwp-studio/src/main.ts`
  - `updateLoadProgress()`를 추가하여 상태창에 `파일 로딩 NN% - 단계` 형식으로 표시한다.
  - `waitForNextPaint()`에 timeout fallback을 두어 background/tab throttling 상황에서도 로딩이 멈추지 않게 했다.
  - 파일 선택 경로는 `0% 파일 읽는 중`부터 시작하고, URL/postMessage/autosave처럼 바이트를 이미 받은 경로도 `loadBytes()`에서 `0% 문서 데이터 준비 중`부터 시작한다.
  - `wasm.loadDocument()` 직전 `25% 문서 파싱 및 쪽 계산 중`을 표시한다.
- `src/document_core/queries/cursor_rect.rs`
  - 초기 커서 rect 탐색에서 `build_page_tree()` 대신 `build_page_tree_cached()`를 사용한다.
- `rhwp-studio/src/engine/input-handler*.ts`
  - 표 resize hover bbox 조회 시 현재 page hint를 전달한다.
  - page hint에 따라 bbox 범위가 달라질 수 있어 hover cache key에 `pageHint`를 포함했다.

## 검증 계획

- `rhwp-studio` TypeScript 빌드 확인.
- 필요 시 WASM 빌드 후 Studio에서 `samples/issue1949_giant_cell_nested_tables_perf.hwp` 로딩 상태창과 콘솔 long task 변화를 확인한다.

## 검증 결과

- `npm run build` (`rhwp-studio`) 통과.
- `cargo fmt --check` 통과.
- `env CARGO_INCREMENTAL=0 cargo check --all-targets` 통과.
- `wasm-pack build --target web --out-dir pkg` 통과.
- 갱신된 `pkg` 기준으로 `npm run build` 재통과.
- `npm test` (`rhwp-studio`) 166개 통과.
- `git diff --check` 통과.
- Vite dev server: `http://127.0.0.1:7701/`
- CORS static server: `http://127.0.0.1:8765/`
- URL 자동 로드 검증:
  - 샘플: `samples/issue1949_giant_cell_nested_tables_perf.hwp`
  - 상태창 샘플링 결과:
    - `파일 로딩 25% - 문서 파싱 및 쪽 계산 중...`
    - `파일 로딩 100% - 완료`
    - `issue1949_giant_cell_nested_tables_perf.hwp — 115페이지 (1558.3ms)`
  - 최종 상태:
    - 페이지: `1 / 115 쪽`
    - 구역: `구역: 1 / 1`
    - 상태창: `issue1949_giant_cell_nested_tables_perf.hwp — 115페이지 (1558.3ms)`

## 남은 주의점

- `wasm.loadDocument()` 자체는 여전히 동기 호출이므로 내부 byte/paragraph 단위의 실제 진행률은 아니다.
- 현재 구현은 UI 체감 개선을 위한 단계 기반 진행률이다.
- 실제 내부 진행률이 필요하면 Rust/WASM progress callback 또는 worker/chunked loading 구조가 별도 후속으로 필요하다.
