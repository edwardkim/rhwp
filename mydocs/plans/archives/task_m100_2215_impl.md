# 구현 계획서 — Task M100 #2215

## 1. 목적과 승인 경계

- 이슈: [#2215](https://github.com/edwardkim/rhwp/issues/2215)
- 브랜치: `issue-2215-selection-page-range`
- 수행 계획서: `mydocs/plans/task_m100_2215.md`
- 근거 보고서:
  - `mydocs/working/task_m100_2215_stage1.md`
  - `mydocs/working/task_m100_2215_stage2.md`
- 기준: `upstream/devel@eb9c7f1f`
- 작성일: 2026-07-19

115쪽 거대 셀의 마우스 드래그 선택에서 endpoint page를 이미 알고 있는데도 외부 표가
걸친 115개 page tree를 매 갱신마다 모두 구축하는 경로를 제거한다. 정상 pointer drag는
endpoint 사이의 host page만 조회하고 shared page-tree cache를 사용한다. page hint가 없거나
유효하지 않으면 기존 전체 탐색으로 정확성을 우선 복구한다.

이 문서는 구현 전 승인 대상이다. 승인 전에는 아래 production source와 테스트를 수정하지
않는다.

## 2. 확정 설계

### 2.1 API 호환 계약

기존 positional `getSelectionRectsInCell(...)`의 인자와 동작은 유지한다. 기존 options API인
`getSelectionRectsInCellEx(optionsJson)`에 다음 optional key만 추가한다.

```ts
startPageHint?: number
endPageHint?: number
```

- 두 key가 모두 있을 때만 hinted path를 시도한다.
- 하나라도 없으면 처음부터 기존 전체 host-page path를 사용한다.
- key가 정수가 아니거나 유효한 host page에 속하지 않으면 전체 path로 fallback한다.
- public positional signature를 바꾸지 않으므로 기존 JavaScript/TypeScript 호출자는 깨지지
  않는다.
- `*Ex`는 options object 확장이라는 기존 #1413 규약을 따른다.

Studio의 `WasmBridge.getSelectionRectsInCell()`에는 마지막 optional options를 추가한다.

```ts
type SelectionPageHints = {
  startPageHint: number;
  endPageHint: number;
};
```

두 값이 있으면 `getSelectionRectsInCellEx()`를 호출하고, 없으면 기존 positional API를
호출한다. `InputHandler.updateSelection()`은 추가 hit test 없이 각 endpoint가 이미 보존한
`cursorRect?.pageIndex`를 전달한다. 키보드 선택처럼 page 정보가 없는 경로는 기존 fallback을
그대로 사용한다.

### 2.2 native 후보 페이지 계약

`src/document_core/queries/cursor_nav.rs`에 page 후보 계산을 selection rect 계산과 분리한 작은
pure helper를 둔다. 개념적 반환은 다음 두 상태다.

```rust
enum SelectionPagePlan {
    Hinted(Vec<u32>),
    FullFallback(Vec<u32>),
}
```

후보 계산 규칙:

1. 기존 `find_pages_for_paragraph(section_idx, parent_para_idx)`의 host page 목록을 권위
   집합으로 얻는다.
2. 두 hint가 모두 host page 목록에 있어야 한다.
3. 유효하면 `min(start, end)..=max(start, end)`에 속하는 host page만 보존한다.
4. same-page는 정확히 1장, cross-page는 두 endpoint 사이의 host page가 된다.
5. 누락·범위 밖·빈 결과이면 전체 host page 목록을 선택한다.

정상 hinted 후보는 `build_page_tree_cached()`로 구성한다. 먼저 기존 함수 구조를 최소 변경해
후보 page tree만 로컬에 보관한다. warm 1-page 경로에서 tree clone이 p95 기준을 방해하면
같은 구현 단계 안에서 `with_page_tree_cached()` 기반의 borrowed query로 전환한다. 115장을
cached clone하며 순회하는 방식은 완료안으로 인정하지 않는다.

### 2.3 정확성 fallback

selection 계산을 다음 두 계층으로 나눈다.

```text
get_selection_rects_native(..., page_hints)
  -> 후보 계획 수립
  -> compute_selection_rects_on_pages(candidate_pages)
  -> endpoint/segment 해소 여부 검사
  -> 실패 시 compute_selection_rects_on_pages(full_host_pages) 재시도
  -> JSON 직렬화
```

내부 pass는 rect 외에 다음 진단을 반환한다.

- 선택의 첫 유효 segment가 rect로 해소됐는지
- 선택의 마지막 유효 segment가 rect로 해소됐는지
- 기대한 non-empty line segment 수와 실제 rect segment 수
- 실제 사용 page 수와 fallback 여부(테스트 assertion용, public WASM 결과에는 노출하지 않음)

non-empty 선택에서 endpoint 또는 필요한 segment를 찾지 못하면 부분 결과를 반환하지 않고
전체 host-page path로 한 번 재시도한다. full fallback의 cursor bias, 줄 끝 처리, rect JSON
형식은 기존 동작을 보존한다.

### 2.4 split paragraph 경계

Stage 3-D 뒤 작업지시자 수동 검증에서 동일 offset이 이전·다음 page fragment 양쪽에 존재하는
실제 cross-page split 선택이 잘못된 rect와 긴 fallback을 함께 만든다는 사실을 확인했다.

- UI 1→2, 56→57, 114→115 경계의 대표 split cell paragraph에서 HWP/HWPX 모두 재현된다.
- 현재 첫-hit 탐색은 경계 offset의 이전-page trailing을 먼저 고른 뒤 다음-page right hit의
  x를 같은 rect에 섞어 페이지 폭을 최대 393.4px 초과한다.
- 선택 segment가 충분히 해소되지 않으면 115장 full fallback으로 돌아가 최대 26.99초가
  소요됐다.
- 이는 단순한 기존 정확성 잔여가 아니라 #2215의 hinted candidate 경로가 유효한 두 page를
  받았는데도 잘못된 rect를 반환하고 성능 목표를 깨는 blocker다.

Stage 3-E에서는 line segment마다 left/right cursor를 **같은 page tree 안에서 짝지은 후보**로
계산한다. 경계 offset이 두 fragment에 존재하면 해당 segment의 양 endpoint를 함께 해소하는
page를 선택하고, 서로 다른 page 좌표로 하나의 rect 폭을 계산하지 않는다. 후보 page 순서는
selection 진행 방향에 맞춰 단조롭게 유지한다. 프런트엔드 page clipping이나 마지막 페이지
특례는 사용하지 않는다.

## 3. 단계별 구현

### Stage 3-A — native RED와 후보 helper

대상:

- `src/document_core/queries/cursor_nav.rs`
- `tests/issue_2215_selection_page_range.rs` 신규

작업:

1. pure candidate helper와 unit test를 먼저 추가한다.
2. 다음 결정적 계약을 RED로 고정한다.
   - host 115장 + same-page hints → 후보 1장
   - p54→p55 hints → 후보 2장
   - 역방향 hints → 동일한 정렬 후보
   - missing/한쪽만 존재/host 밖 hint → full 115장
3. HWP/HWPX 픽스처에서 기존 전체 탐색 oracle을 고정한다.
   - p5 0..10: rect/copy SHA-256
   - p1250 0..1: rect/copy SHA-256
   - p1250:0→p1275:1: rect 45개, p54–55, rect/copy SHA-256
4. split paragraph의 다음-page same-page 선택을 RED로 둔다.
   - p17 166..170 → p1
   - p1277 78..82 → p56
   - p2499 114..118 → p114
   - rect가 해당 page의 폭을 벗어나지 않음
5. 기존 `tests/issue_658_text_selection_rects.rs`를 함께 실행한다.

Stage 3-A 확인 결과 native와 WASM은 첫 rect 폭에서 0.3px metric 차이가 있었다. 따라서
native test는 페이지·rect 수·대표 좌표 ±0.5px와 HWP/HWPX BLAKE3 동등성을 고정하고,
Stage 2의 SHA-256 byte oracle은 WASM/Studio E2E에서 유지한다.

Stage 3-A에서 production 동작은 아직 바꾸지 않고 RED의 실패 이유와 oracle만 보고서에
기록한다.

산출물:

```text
mydocs/working/task_m100_2215_stage3.md
```

### Stage 3-B — native candidate 제한과 fallback GREEN

대상:

- `src/document_core/queries/cursor_nav.rs`
- `src/wasm_api.rs`

작업:

1. `get_selection_rects_native()`에 내부 optional page hints를 추가한다.
2. 계산 본체를 explicit page 목록을 받는 내부 pass로 분리한다.
3. 정상 hinted path에서는 제한된 page만 cached build한다.
4. endpoint/segment 불충분 시 full host pages로 한 번만 재시도한다.
5. positional API와 `Ex` without hints는 기존 전체 path를 유지한다.
6. `getSelectionRectsInCellEx`가 두 optional key를 파싱해 native로 전달한다.
7. 테스트 전용 진단은 Rust 내부 결과에만 두고 public JSON/API에는 추가하지 않는다.

검증:

```text
cargo test --lib selection_page
cargo test --test issue_2215_selection_page_range -- --nocapture
cargo test --test issue_658_text_selection_rects
```

실제 unit test module 이름은 구현 시 기존 모듈 구조에 맞추되 위 계약을 모두 포함한다.

### Stage 3-C — Studio 전달 경로와 단위 테스트

대상:

- `rhwp-studio/src/core/wasm-bridge.ts`
- `rhwp-studio/src/engine/input-handler.ts`
- `rhwp-studio/tests/selection-page-hints.test.ts` 신규

작업:

1. bridge의 cell selection wrapper에 optional hints를 추가한다.
2. 두 hints가 모두 있을 때만 `Ex` options JSON을 사용한다.
3. `InputHandler.updateSelection()`이 ordered start/end 각각의
   `cursorRect?.pageIndex`를 전달한다.
4. keyboard selection 등 한 hint라도 없는 경로는 positional fallback을 사용한다.
5. 다음 bridge 단위 테스트를 추가한다.
   - 두 hints → `getSelectionRectsInCellEx` 1회, key/value 정확
   - hints 없음 → positional 1회
   - 한 hint만 있음 → positional 1회
   - 반환 JSON parsing과 기존 인자 순서 유지

`input-handler-mouse.ts`의 rAF/auto-scroll 정책은 바꾸지 않는다. hints가 이미 보존되는
`CursorState` 계약을 소비하는 데 그친다.

### Stage 3-D — 실제 pointer drag E2E

대상:

- `rhwp-studio/e2e/selection-page-range-issue2215.test.mjs` 신규
- `rhwp-studio/e2e/MANIFEST.md`
- `rhwp-studio/package.json`
- 필요 시 `.github/workflows/render-diff.yml`의 `node --check` 목록

작업:

1. 저장소 Puppeteer/CDP 하니스로 HWP와 HWPX 115쪽 샘플을 실제 앱 경로로 연다.
2. page layer의 stable source와 cursor rect로 첫·중간·후반 target의 화면 좌표를 얻되,
   선택 자체는 `page.mouse.move/down/move/up`으로 수행한다.
3. WASM bridge method를 test page에서 감싸 다음을 기록한다.
   - positional/Ex 호출 수
   - 전달된 start/end page hints
   - 각 selection rect 호출 시간
   - pagination/Canvas refresh 호출 수
4. 각 same-page drag에서 다음을 assertion한다.
   - focus offset이 pointer 방향으로 진행
   - visible selection highlight가 존재
   - mouseup 후 선택 상태 유지
   - copy 문자열 정확
   - normal hinted path에서 positional fallback 0회
5. split paragraph의 다음-page same-page drag에서 highlight page가 실제 pointer page와
   일치하는지 확인한다.
6. p54→p55 cross-page drag는 기존 45 rect/copy oracle과 자동 스크롤 무회귀를 확인한다.
7. 기존 `drag-selection-autoscroll.test.mjs`를 별도로 재실행한다.

신규 E2E는 MANIFEST에 `상시/active`로 등록하고 `e2e:issue-2215` npm script를 추가한다.
CI의 결정적 gate는 Rust 후보 수·fallback 테스트와 Studio unit test로 둔다. 브라우저 E2E는
스크립트 syntax check를 CI에 연결하고 실제 115쪽 pointer run은 로컬 통합 검증에서 수행한다.
환경별 wall-clock 차이를 CI pass/fail에 직접 사용하지 않는다.

### Stage 3-E — split cell paragraph cross-page 보정

대상:

- `src/document_core/queries/cursor_nav.rs`
- `tests/issue_2215_selection_page_range.rs`
- `rhwp-studio/e2e/selection-page-range-issue2215.test.mjs`
- `mydocs/working/task_m100_2215_stage3.md`

작업:

1. UI 1→2, 56→57, 114→115의 같은 cell paragraph cross-page 범위를 HWP/HWPX RED로
   추가한다.
2. 각 line segment의 leading/trailing cursor pair를 동일 page 후보 안에서 계산한다.
3. segment page가 선택 방향을 거슬러 이전 fragment로 돌아가지 않도록 단조 순서를 지킨다.
4. 서로 다른 page의 x 좌표가 하나의 rect에 결합되는 경우를 구조적으로 차단한다.
5. 모든 rect가 해당 page 폭 안에 있고 기대 page 양쪽을 포함하는지 검증한다.
6. 기존 same-page split, p54→p55 서로 다른 문단 cross-page, stale/missing hint fallback과
   #658 oracle을 함께 유지한다.
7. Studio 실제 pointer drag에서 UI 114→115 highlight, mouseup 유지, copy, callback 시간과
   full fallback 부재를 재검증한다.

이 단계는 line break, pagination, Canvas refresh, renderer clipping을 변경하지 않는다. 동일
page cursor pair만으로 해소되지 않아 LineSeg 또는 layout semantic 변경이 필요하면 구현을
중단하고 다시 승인받는다.

### Stage 4 — 성능·통합 검증과 보고

로컬 browser E2E는 앱 내 브라우저 런타임의 유무에 의존하지 않고 저장소 headless
Puppeteer/CDP 하니스로 수행한다. warm-up 후 HWP/HWPX 각각 첫·중간·후반 same-page
drag를 반복해 다음을 기록한다.

| 지표 | 완료 기준 |
|------|-----------|
| 정상 same-page 후보 page | 1 |
| p54→p55 후보 page | 2 |
| 정상 hinted fallback | 0회 |
| selection callback p95 | 50ms 미만 |
| 반복 long task(50ms 이상) | 0건 |
| drag 중 pagination | 0회 |
| drag 중 Canvas page refresh | 0회 |

wall-clock은 로컬 성능 gate이며, 결정적 후보·fallback assertion과 함께 결과를 보고한다.

## 4. 예상 변경 파일

| 파일 | 변경 |
|------|------|
| `src/document_core/queries/cursor_nav.rs` | 후보 helper, page-scoped pass, cached tree, fallback 및 내부 진단 |
| `src/wasm_api.rs` | `getSelectionRectsInCellEx` optional page hints 파싱 |
| `tests/issue_2215_selection_page_range.rs` | HWP/HWPX oracle, same/cross/split/fallback 회귀 |
| `rhwp-studio/src/core/wasm-bridge.ts` | optional hints를 Ex에 전달하는 호환 wrapper |
| `rhwp-studio/src/engine/input-handler.ts` | ordered endpoint page hint 전달 |
| `rhwp-studio/tests/selection-page-hints.test.ts` | bridge dispatch와 fallback 단위 테스트 |
| `rhwp-studio/e2e/selection-page-range-issue2215.test.mjs` | 실제 pointer drag 통합·성능 회귀 |
| `rhwp-studio/e2e/MANIFEST.md` | 신규 E2E 권위 목록 등록 |
| `rhwp-studio/package.json` | `e2e:issue-2215` script |
| `.github/workflows/render-diff.yml` | 조건부 E2E syntax check 배선 |

`rhwp-studio/public/rhwp.js`, `rhwp.d.ts`, `rhwp_bg.wasm.d.ts`는 public WASM 함수 signature가
바뀌지 않으므로 수동 수정하지 않는다. WASM binary는 빌드 검증 산출물이며 저장소의 ignore
정책을 따른다.

## 5. 검증 순서

구현 단계별 표적 검증:

```bash
cargo fmt --check
cargo test --test issue_2215_selection_page_range -- --nocapture
cargo test --test issue_658_text_selection_rects
npm --prefix rhwp-studio test
python3 scripts/check_e2e_manifest.py
node --check rhwp-studio/e2e/selection-page-range-issue2215.test.mjs
```

WASM/Studio 통합 검증:

```bash
docker compose run --rm wasm
cp pkg/rhwp.js rhwp-studio/public/
cp pkg/rhwp_bg.wasm rhwp-studio/public/
npm --prefix rhwp-studio run build
npm --prefix rhwp-studio run e2e:issue-2215 -- --mode=headless
npm --prefix rhwp-studio run e2e:drag-autoscroll -- --mode=headless
```

위 copy는 로컬 브라우저가 방금 빌드한 WASM을 읽게 하는 검증 단계다. tracked
`rhwp-studio/public/rhwp.js`의 생성 diff는 source 변경으로 커밋하지 않고 검증 뒤 제외한다.

focused 결과 공유 후 작업지시자의 별도 승인을 받아 PR 전 전체 CI 성격 검증을 수행한다.

```bash
cargo test --verbose
cargo clippy --all-targets -- -D warnings
```

최종 결과는 다음 문서에 기록한다.

```text
mydocs/report/task_m100_2215_report.md
```

## 6. 중단·재승인 조건

다음 중 하나가 발생하면 임의로 범위를 확대하지 않고 작업을 중단해 결과를 공유한다.

1. 정상 mouse drag에서 endpoint page hint를 얻을 수 없다.
2. same-page 1장 또는 cross-page inclusive 후보에서 정상 drag가 반복적으로 fallback한다.
3. 정확성을 위해 pagination, Canvas refresh, line break, clip 또는 좌표 산식을 바꿔야 한다.
4. 동일 page cursor pair 선택만으로 해소되지 않고 LineSeg·layout·clip semantic 변경이
   필요하다.
5. cached 1-page tree clone을 제거해도 p95 50ms를 만족하지 못하고 별도 hit-test/overlay 최적화가
   필요하다.
6. 기존 positional/Ex without hints, #658 또는 Stage 2 rect/copy oracle이 회귀한다.

## 7. 완료 조건

- 정상 same-page pointer drag가 후보 1장과 cached tree만 사용한다.
- 정상 cross-page drag가 endpoint 사이의 host page만 사용한다.
- 정상 hinted drag에서 full fallback이 발생하지 않는다.
- missing/invalid/stale hint는 기존 전체 탐색과 같은 정확한 결과로 fallback한다.
- HWP/HWPX의 비분할 rect/copy oracle과 #658 회귀가 유지된다.
- 다음-page same-page split selection이 실제 pointer page에 rect를 반환한다.
- 같은 cell paragraph의 cross-page split selection이 양쪽 page에 rect를 반환하고 각 rect가
  해당 page 폭 안에 있다.
- 실제 pointer drag에서 highlight, mouseup selection, copy, auto-scroll이 유지된다.
- 문서화된 warm 조건에서 callback p95 50ms 미만, long task 0건이다.
- drag 중 pagination과 Canvas refresh 호출이 0회다.
- 정상 hinted split cross-page drag가 full host-page fallback 없이 endpoint page 범위만
  사용한다.
