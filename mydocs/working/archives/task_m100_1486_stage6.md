# Task M100 #1486 Stage 6: rhwp-studio HWPX 경고 상태 누수 보정

- 이슈: #1486
- 브랜치: `local/task_m100_1486`
- 작성일: 2026-06-24
- 방법론: Hyper-Waterfall
- 선행 커밋:
  - `a2a7d848 task 1486: 마지막 쪽 TAC 그림과 RowBreak 회귀 보정`

## 배경

작업지시자가 `samples/hwpx_sample2.hwpx`를 rhwp-studio에서 열면 `HWPX 비표준 감지` 모달이 표시되고,
그 창을 닫은 뒤 메뉴에서 새 파일을 눌러도 새 문서임에도 같은 모달이 다시 표시된다고 보고했다.

추가로 같은 HWPX 문서에서 `자동 보정 (권장)`을 누르면 로드 직후 29쪽이던 문서가 30쪽으로
늘어나는 현상이 확인되었다.

## 재현 흐름

1. rhwp-studio에서 `samples/hwpx_sample2.hwpx`를 연다.
2. `HWPX 비표준 감지` 모달을 닫거나 그대로 보기로 닫는다.
3. 메뉴에서 새 파일을 선택한다.
4. 새 문서에도 이전 HWPX validation warning 모달이 다시 표시된다.

## 분석 계획

- `rhwp-studio/src/main.ts`의 `initializeDocument()`에서 validation modal을 띄우는 조건을 확인한다.
- `rhwp-studio/src/core/wasm-bridge.ts`의 `createNewDocument()`가 기존 WASM 문서 객체를 재사용할 때
  validation report가 초기화되는지 확인한다.
- Rust `HwpDocument::createBlankDocument()`와 document core 초기화 경로에서 validation warning 상태가
  새 문서에 남는지 테스트로 재현한다.

## 수정 방향

- 새 문서 생성 시 이전 HWPX validation warning 상태가 남지 않도록 WASM 문서 상태를 초기화한다.
- 가능하면 WASM/Rust 단위 테스트로 `HWPX 로드 후 createBlankDocument() → warning count 0`을 고정한다.
- rhwp-studio 쪽에서는 필요 시 `sourceFormat === 'hwpx'` 조건을 추가해 HWPX 경고 모달이 HWP 새 문서에는
  표시되지 않도록 방어한다.
- `LinesegTextRunReflow`는 경고는 유지하되 자동 보정 대상에서 제외한다. 이 유형은 한컴이 계산한 1개
  lineseg를 강제로 다시 풀면 페이지 수가 바뀔 수 있다.

## 검증 계획

- Rust 단위/통합 테스트: HWPX 경고 샘플 로드 후 새 문서 생성 시 validation warning 0건 확인
- rhwp-studio TypeScript 검사 또는 기존 테스트 중 관련 범위 확인
- 필요 시 브라우저/Playwright로 `HWPX 열기 → 모달 닫기 → 새 문서` 흐름 확인

## 구현 결과

- `src/document_core/commands/document.rs`
  - `create_blank_document_native()`에서 새 HWP 템플릿을 로드한 뒤 `source_format`을 `Hwp`로 되돌리고
    `validation_report`를 빈 리포트로 초기화했다.
  - `reflow_linesegs_on_demand()`의 broad reflow 대상에서 `LinesegTextRunReflow` 패턴을 제외했다.
    `LinesegArrayEmpty`와 `LinesegUncomputed`처럼 명백한 미계산 상태만 자동 보정한다.

- `src/wasm_api/tests.rs`
  - `samples/hwpx_sample2.hwpx` 로드 후 `createBlankDocument()`를 호출하면 warning count가 0이고
    source format이 `hwp`로 돌아오는 회귀 테스트를 추가했다.
  - 같은 샘플에서 `reflowLinesegs()`를 호출해도 `LinesegTextRunReflow` 151건은 자동 보정하지 않고
    페이지 수가 29쪽 그대로 유지되는 테스트를 추가했다.

- `rhwp-studio/src/main.ts`
  - validation modal 확인을 `wasm.getSourceFormat() === 'hwpx'`일 때만 수행하도록 방어했다.
  - 자동 보정 결과가 0건이면 렌더 재계산과 dirty 표시를 하지 않도록 했다.

## 검증 결과

- `cargo test --release --lib test_create_blank_document_clears_previous_hwpx_validation_warnings -- --nocapture`
  - 통과: 1 passed
- `cargo test --release --lib test_reflow_linesegs_keeps_hwpx_sample2_page_count_for_textrun_warnings -- --nocapture`
  - 통과: 1 passed
- `cargo test --release --lib needs_reflow_broadly_skips_textrun_reflow -- --nocapture`
  - 통과: 1 passed
- `cargo test --release --lib validate_detects_textrun_reflow_pattern -- --nocapture`
  - 통과: 1 passed
- `wasm-pack build --target web --out-dir pkg`
  - 통과
- `node` + 최신 `pkg/rhwp.js` 직접 확인
  - `loaded`: pageCount 29, source `hwpx`, warnings 151
  - `after-reflow`: pageCount 29, reflowed 0, warnings 151
  - `after-blank`: pageCount 1, source `hwp`, warnings 0
- `npm run build` (`rhwp-studio`)
  - 통과

## 참고

네이티브 CLI 기준 `samples/hwpx_sample2.hwp`와 `samples/hwpx_sample2.hwpx`는 모두 29쪽으로 확인했다.
자동 보정 전에는 최신 `pkg` 기준도 두 파일 모두 29쪽이다.
