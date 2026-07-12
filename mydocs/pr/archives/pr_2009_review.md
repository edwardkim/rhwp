# PR #2009 리뷰

## 메타

| 항목 | 내용 |
|------|------|
| PR | #2009 |
| 제목 | Studio 파일 로딩 진행률 표시 및 초기 병목 완화 |
| 작성자 | jangster77 |
| base | `devel` |
| head | `task_m100_2008_studio_load_progress` |
| 관련 이슈 | #2008 |
| 문서 작성 시점 참고값 | mergeable: MERGEABLE, mergeStateStatus: BLOCKED(CI 진행 중) |

## 변경 범위

- `rhwp-studio/src/main.ts`
  - 상태창(`#sb-message`)에 `파일 로딩 NN% - 단계` 형식의 단계 기반 진행률 표시를 추가했다.
  - `loadBytes()` 공통 경로에 진행 상태를 두어 파일 선택, URL 로드, postMessage, autosave 복구 경로가 같은 표시를 사용한다.
  - 긴 동기 작업 전에 `requestAnimationFrame` + timeout fallback 으로 paint 기회를 준다.
- `src/document_core/queries/cursor_rect.rs`
  - 초기 cursor rect 탐색에서 `build_page_tree_cached()`를 사용한다.
- `rhwp-studio/src/engine/input-handler.ts`, `input-handler-mouse.ts`
  - 표 resize hover bbox 조회에 현재 page hint 를 넘긴다.
  - page hint 가 다른 bbox cache 를 오염시키지 않도록 `cachedTableRef`에 `pageHint`를 포함했다.
- 운영 기록
  - `mydocs/working/task_m100_2008_stage1.md`
  - `mydocs/orders/20260707.md`

## 렌더 영향 및 시각 검증 판정

렌더 출력 자체를 고치는 PR 이 아니라 Studio 로딩 상태 표시와 로드 직후 interaction 병목 완화가 목적이다.
PDF/SVG 기준 시각 sweep 은 merge blocker 로 보지 않는다.

대신 실제 Studio 브라우저 로드 검증을 수행했다.

- 샘플: `samples/issue1949_giant_cell_nested_tables_perf.hwp`
- 확인 경로: Vite dev server + URL 자동 로드
- 상태창 샘플링:
  - `파일 로딩 25% - 문서 파싱 및 쪽 계산 중...`
  - `파일 로딩 100% - 완료`
  - `issue1949_giant_cell_nested_tables_perf.hwp — 115페이지 (1558.3ms)`
- 최종 페이지 상태:
  - `1 / 115 쪽`
  - `구역: 1 / 1`

## 로컬 검증

- `npm run build` (`rhwp-studio`) 통과
- `cargo fmt --check` 통과
- `env CARGO_INCREMENTAL=0 cargo check --all-targets` 통과
- `wasm-pack build --target web --out-dir pkg` 통과
- 갱신된 `pkg` 기준 `npm run build` 재통과
- `npm test` (`rhwp-studio`) 166개 통과
- `git diff --check` 통과

## 리스크

- 진행률은 byte/paragraph 단위의 실제 내부 진행률이 아니라 Studio 단계 기반 진행률이다.
- `wasm.loadDocument()`는 여전히 동기 호출이므로 내부 파싱 중에는 25% 단계가 길게 유지될 수 있다.
- 정확한 내부 진행률은 Rust/WASM progress callback 또는 worker/chunked loading 구조가 필요하다.

## 결론

PR 내용과 #2008 수용 기준에 맞는 1차 개선으로 판단한다.

merge 전 최종 조건:

- PR head 최신 커밋 기준 GitHub Actions 통과
- `devel` 기준 merge 가능 상태 확인
- 작업지시자 승인

