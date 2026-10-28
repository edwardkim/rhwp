# 단계별 완료 보고서 — Task M100 #2215 Stage 3-A/3-B/3-C

## 1. 결론

GREEN 구현 전에 page 후보 계약과 native/WASM 경계의 RED를 고정했다.

- 115장 host에서 same-page hint는 1장만 선택한다.
- p54→p55와 역방향 p55→p54는 `[54, 55]` 두 장만 선택한다.
- missing, 한쪽만 존재, host 밖 hint는 115장 full fallback을 유지한다.
- HWP/HWPX의 정상 same-page·cross-page rect/copy oracle은 유지된다.
- 기존 #658 selection rect 2건은 계속 GREEN이다.
- `getSelectionRectsInCellEx`가 아직 page hints를 소비하지 않으므로 split paragraph의
  다음-page same-page 선택 6건은 의도대로 RED다.

Stage 3-A에서는 후보 helper를 실제 selection 계산에 연결하지 않았다. 따라서 production
selection 동작과 성능은 아직 바뀌지 않았다.

## 2. 추가한 후보 계약

`src/document_core/queries/cursor_nav.rs`에 다음 내부 계획을 추가했다.

```text
SelectionPagePlan::Hinted(Vec<u32>)
SelectionPagePlan::FullFallback(Vec<u32>)
```

`plan_selection_pages()`는 host page 목록과 optional start/end hint만 받아 결과를 만드는 pure
helper다. production 계산과 분리되어 있으며 unit test 4건으로 다음을 고정했다.

| 입력 | 결과 |
|------|------|
| host `0..114`, hints `54,54` | `Hinted([54])` |
| host `0..114`, hints `54,55` | `Hinted([54,55])` |
| host `0..114`, hints `55,54` | `Hinted([54,55])` |
| missing/한쪽만 존재/`999` | `FullFallback(0..114)` |
| sparse host `[2,4,9,10]`, hints `4,10` | `Hinted([4,9,10])` |

검증 결과:

```text
running 4 tests
4 passed; 0 failed
```

## 3. 정상 selection oracle

신규 `tests/issue_2215_selection_page_range.rs`는 HWP/HWPX 각각 다음 범위를 기존
`getSelectionRectsInCellEx`로 조회한다.

| 범위 | native 계약 |
|------|-------------|
| p5 0..10 | rect 1개 / p0, 대표 좌표 ±0.5px, copy `1.1.1 수면비행` |
| p1250 0..1 | rect 1개 / p54, 대표 좌표 ±0.5px, copy `8` |
| p1250:0→p1275:1 | rect 45개 / p54–55, copy 1,517자와 prefix/suffix |

각 rect JSON과 copy text의 BLAKE3를 HWP/HWPX 사이에서 byte-level 비교해 두 형식이 같은
결과임도 고정했다.

Stage 2의 SHA-256은 WASM 실행에서 확보한 값이다. native 실행은 첫 rect 폭이 WASM 111.7px,
native 112.0px로 0.3px 달랐다. 이는 native/WASM font metric 실행 대상 차이이므로 native
테스트에서 WASM JSON을 byte-level로 강제하지 않았다.

- native: 구조·페이지·좌표 ±0.5px와 HWP/HWPX byte 동등성
- WASM/Studio: Stage 2의 기존 SHA-256 oracle

이렇게 대상별 oracle을 분리해 실제 회귀를 잡으면서 플랫폼 metric 차이로 인한 가짜 실패를
피한다.

검증 결과:

```text
issue_2215_hwp_and_hwpx_preserve_normal_selection_oracles ... ok
1 passed; 0 failed; finished in 193.73s
```

현재 full host-page 탐색이 유지되어 이 RED 단계의 native test가 오래 걸린다. Stage 3-B
GREEN 뒤 hinted path가 연결되면 같은 테스트 시간이 후보 page 수에 맞게 줄어야 한다.

## 4. split paragraph RED

기존 `getSelectionRectsInCellEx` options에 `startPageHint`와 `endPageHint`를 넣었지만 현재
WASM adapter가 두 key를 무시하므로 전체 115쪽의 첫 fragment를 선택한다.

| 형식 | cell paragraph | 기대 | 실제 | 결과 |
|------|---------------:|------|------|------|
| HWP | 17 / 166..170 | p1, page 폭 안 | p0, x=670.9, width=516.2 | RED |
| HWP | 1277 / 78..82 | p56, page 폭 안 | p55, x=670.6, width=473.7 | RED |
| HWP | 2499 / 114..118 | p114, page 폭 안 | p113, x=670.6, width=463.4 | RED |
| HWPX | 17 / 166..170 | p1, page 폭 안 | p0, x=670.9, width=516.2 | RED |
| HWPX | 1277 / 78..82 | p56, page 폭 안 | p55, x=670.6, width=473.7 | RED |
| HWPX | 2499 / 114..118 | p114, page 폭 안 | p113, x=670.6, width=463.4 | RED |

모든 실제 rect는 page width 793.7px를 벗어난다. Stage 2의 기존 원인과 일치하며 Stage 3-B가
해결해야 할 정확한 GREEN 목표다.

## 5. 기존 회귀

```text
cargo test --test issue_658_text_selection_rects
2 passed; 0 failed; finished in 0.10s
```

`cargo fmt --check`도 통과했다.

## 6. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/document_core/queries/cursor_nav.rs` | page 후보 pure helper와 unit test 4건 |
| `tests/issue_2215_selection_page_range.rs` | HWP/HWPX 정상 oracle 및 split RED |
| `mydocs/working/task_m100_2215_stage3.md` | Stage 3-A 결과 기록 |

`src/wasm_api.rs`, Studio source, page-tree build 경로는 아직 변경하지 않았다.

## 7. Stage 3-A 당시 다음 승인 단계

Stage 3-B에서는 승인된 구현계획에 따라 다음만 수행한다.

1. `get_selection_rects_native()`가 optional page hints를 받도록 내부 경계를 확장한다.
2. candidate page 목록만 cached build하는 pass를 분리한다.
3. endpoint/segment 미해소 시 full host-page fallback한다.
4. `getSelectionRectsInCellEx`가 두 optional hint를 파싱해 전달한다.
5. 이 보고서의 6개 split RED를 GREEN으로 전환한다.

Studio 전달과 실제 pointer E2E는 Stage 3-C/3-D이며 Stage 3-B에 섞지 않는다.

---

## 8. Stage 3-B GREEN 결론

2026-07-19 최신 `upstream/devel@af5902b6`에 RED 커밋을 rebase한 뒤 native candidate 제한과
정확성 fallback을 구현했다.

- `getSelectionRectsInCellEx`가 optional `startPageHint`/`endPageHint`를 파싱한다.
- 두 hint가 유효하면 same-page 1장 또는 endpoint 사이 host page만 조회한다.
- hinted 후보만 shared `build_page_tree_cached()`를 사용한다.
- 필요한 line segment를 모두 해소하지 못하면 기존 full host-page path로 한 번 재시도한다.
- positional, missing, one-sided, invalid hint의 full fallback은 기존 uncached 함수 로컬 tree
  수명을 유지한다.
- Stage 3-A의 split RED 6건은 모두 GREEN으로 전환됐다.

Studio source는 아직 변경하지 않았으므로 실제 mouse drag는 계속 positional API를 사용한다.
이번 GREEN은 native/WASM options 경계까지만 완료한 상태다.

## 9. native 구현

`get_selection_rects_native()`에 내부 `Option<(u32, u32)>` page hints를 추가했다.

```text
host pages 조회
→ plan_selection_pages()
→ Hinted: 제한된 page만 shared cache로 build
→ FullFallback: 기존 전체 page를 함수 로컬 tree로 build
→ line segment별 rect 계산
→ expected_segments != rendered_segments이면 hints 없이 full retry
```

fallback 판단은 non-empty line segment 수와 실제 rect segment 수를 비교한다. 따라서 hinted
범위가 일부 rect만 반환하는 경우 부분 결과를 노출하지 않는다. page hints가 없거나 host
집합 밖이면 처음부터 full fallback을 사용하므로 재귀는 한 번을 넘지 않는다.

full fallback까지 shared cache를 사용하면 115개 tree가 호출 뒤에도 남아 메모리 체류가
증가할 수 있다. 그래서 cache는 정상 hinted 후보에만 적용하고 positional/fallback은 기존
수명을 보존했다.

## 10. WASM options 호환

`src/wasm_api.rs`의 기존 positional API는 `page_hints=None`으로 호출한다. `Ex`만 두 optional
key를 `zip()`해 둘 다 있을 때 native hinted path로 전달한다.

| 호출 | 동작 |
|------|------|
| positional | 기존 full host-page 탐색 |
| `Ex` hints 없음 | 기존 full host-page 탐색 |
| `Ex` 한 hint만 존재 | 기존 full host-page 탐색 |
| `Ex` host 밖 hints | 기존 full host-page 탐색 |
| `Ex` 유효 same-page hints | endpoint page 1장 |
| `Ex` 유효 cross-page hints | endpoint 사이 host pages |
| `Ex` 유효하지만 endpoint miss | full host-page 재시도 |

public positional signature와 JSON 반환 형식은 바뀌지 않았다.

## 11. GREEN 검증

### #2215 통합 회귀

```text
cargo test --test issue_2215_selection_page_range -- --nocapture --test-threads=1
3 passed; 0 failed; finished in 99.25s
```

세부 결과:

- HWP/HWPX 정상 same-page·cross-page rect/copy oracle 유지
- HWP의 valid-but-stale p0 hint로 p54 endpoint 조회 시 full fallback 결과 유지
- missing, one-sided, invalid hints가 positional 결과와 동일
- p1, p56, p114 same-page split rect가 HWP/HWPX 모두 기대 page와 page 폭 안에 존재

Stage 3-A full target은 정상 oracle의 초기 과엄격 metric assertion을 포함한 상태에서
238.34초였고, GREEN target은 stale full-fallback 회귀까지 추가하고도 99.25초였다. 이 값에는
두 형식 parse와 의도적인 115-page fallback이 포함되므로 최종 drag p95로 해석하지 않는다.

### 후보 helper와 기존 회귀

```text
cargo test --lib issue_2215_selection_page_plan_tests
4 passed; 0 failed

cargo test --test issue_658_text_selection_rects
2 passed; 0 failed; finished in 0.08s

cargo fmt --check
통과
```

## 12. Stage 3-B 변경 파일

| 파일 | 변경 |
|------|------|
| `src/document_core/queries/cursor_nav.rs` | hinted candidate 연결, 제한된 cached tree, segment fallback |
| `src/wasm_api.rs` | `Ex` optional page hints 파싱과 positional 호환 |
| `tests/issue_2215_selection_page_range.rs` | stale/missing/one-sided/invalid fallback 회귀 추가 |

## 13. 다음 승인 단계

Stage 3-C에서는 다음 Studio 전달만 구현한다.

1. `WasmBridge.getSelectionRectsInCell()`에 optional hints를 추가한다.
2. 두 hints가 있을 때만 `getSelectionRectsInCellEx`를 호출한다.
3. `InputHandler.updateSelection()`이 ordered endpoint의 `cursorRect?.pageIndex`를 전달한다.
4. bridge dispatch 단위 테스트를 추가한다.

mouse rAF, auto-scroll, rendering 또는 pagination 정책은 변경하지 않는다. 실제 pointer drag
E2E와 p95 측정은 Stage 3-D 승인 뒤 수행한다.

---

## 14. Stage 3-C Studio 전달 결론

Studio가 이미 보유한 ordered selection endpoint의 `cursorRect.pageIndex`를 native/WASM의
hinted path까지 전달하도록 최소 경계를 연결했다.

- 시작·끝 endpoint의 page가 모두 있을 때만 `getSelectionRectsInCellEx`를 호출한다.
- hint가 없거나 한쪽뿐이거나 음수이면 기존 positional API를 사용한다.
- 아직 `Ex`를 노출하지 않는 구버전 WASM과 결합해도 positional API로 복구한다.
- `InputHandler`는 same-cell selection에만 ordered start/end page를 전달한다.
- mouse rAF, auto-scroll, selection overlay, pagination 및 Canvas refresh 정책은 변경하지
  않았다.

WASM options의 page key는 기존 Stage 3-B 계약과 동일한 camelCase를 사용한다. JSON 반환은
기존과 같이 `SelectionRect[]`로 파싱하므로 Studio 공개 반환 형식도 바뀌지 않는다.

## 15. Studio 회귀 계약

WASM import alias에 의존하지 않는 pure dispatch helper를 두어 다음 세 경우를 직접 검증했다.

| 조건 | 기대 호출 |
|------|-----------|
| 유효한 start/end page와 `Ex` 존재 | `Ex` 1회, positional 0회 |
| hint 없음·한쪽만 존재·음수 | positional 1회 |
| 유효한 hint지만 구버전 WASM에 `Ex` 없음 | positional 1회 |

검증 결과:

```text
node --test tests/selection-page-hints.test.ts
3 passed; 0 failed

npm test
365 passed; 0 failed

npm run build
TypeScript + Vite production build 통과
```

분리 worktree에는 설치 의존성과 생성된 `pkg`가 없으므로 build 검증 중에만 주 작업공간의
`node_modules`와 `pkg`를 임시 symlink로 재사용했고 검증 직후 제거했다. 생성 산출물과
symlink는 변경 목록에 포함하지 않는다.

## 16. Stage 3-C 변경 파일

| 파일 | 변경 |
|------|------|
| `rhwp-studio/src/core/selection-page-hints.ts` | optional hint 검증, Ex/positional 호환 dispatch |
| `rhwp-studio/src/core/wasm-bridge.ts` | 셀 selection 조회에 optional page hints 연결 |
| `rhwp-studio/src/engine/input-handler.ts` | ordered same-cell endpoint page 전달 |
| `rhwp-studio/tests/selection-page-hints.test.ts` | hinted/fallback/구버전 WASM 단위 회귀 |

## 17. 다음 승인 단계

Stage 3-D에서는 production 정책을 더 바꾸지 않고 실제 pointer drag 검증과 계측만 수행한다.

1. 첫·중간·후반 same-page drag에서 focus offset, visible highlight 및 mouseup 유지 확인
2. page 경계 cross-page drag와 auto-scroll 선택 확인
3. HWP/HWPX 복사 문자열을 기존 oracle과 비교
4. warm selection rAF callback p50/p95, long task, pagination 및 Canvas refresh 호출 수 기록
5. 완료 조건 미달 시 원인을 분리하고 구현 범위 확대 전 다시 승인 요청

---

## 18. Stage 3-D 실제 pointer E2E 결론

최신 worktree WASM을 새로 빌드하고 실제 mouse event 기반으로 HWP/HWPX를 검증했다.

- in-app browser에서 HWP 첫·중간·후반 페이지를 직접 드래그해 하이라이트와 mouseup 후
  선택 유지, clipboard 문자열을 확인했다.
- headless Chrome에서 HWP/HWPX 각각 전·중·후반 same-page를 6회씩 반복했다.
- p54→p55 cross-page는 두 endpoint를 edge 밖에 동시에 표시해 정확한 endpoint와 copy
  oracle을 검증했다.
- 별도 edge drag는 1.2초 동안 auto-scroll을 유지해 실제 rAF 반복 경로를 검증했다.
- 기존 `drag-selection-autoscroll.test.mjs`도 별도로 GREEN을 확인했다.

측정 환경:

```text
macOS arm64
Google Chrome 150.0.7871.128
Node.js v24.15.0
rustc 1.93.1
viewport: same-page/auto-scroll 1280×900
cross-page exact: 1280×3600 (두 endpoint를 edge 구역 밖에 배치)
```

## 19. same-page warm 성능과 정확성

각 반복은 실제 `mouse.down → 8회 mouse.move → mouse.up`으로 수행했다. `drag callback`은
`updateTextSelectionDragFromPointer()` 전체, `rect`는 Studio bridge의
`getSelectionRectsInCell()` 호출 시간을 뜻한다.

| 형식·위치 | drag callback p50 / p95 | rect p95 | copy | warm long task |
|------|------:|------:|------|------:|
| HWP first p0 | 1.0 / 1.2ms | 0.2ms | `1.1.1 수면비행` | 0 |
| HWP middle p54 | 0.7 / 0.9ms | 0.2ms | `8` | 0 |
| HWP late split p114 | 0.5 / 0.9ms | 0.2ms | `설비가 ` | 0 |
| HWPX first p0 | 0.8 / 1.1ms | 0.2ms | `1.1.1 수면비행` | 0 |
| HWPX middle p54 | 0.6 / 0.9ms | 0.2ms | `8` | 0 |
| HWPX late split p114 | 0.5 / 1.0ms | 0.2ms | `설비가 ` | 0 |

모든 36회 반복에서 visible highlight와 mouseup 후 선택이 유지됐다. 실제 Studio 호출에서
관측한 hint도 각 위치의 `[0,0]`, `[54,54]`, `[114,114]`뿐이어서 same-page가 115쪽 full
range로 되돌아가지 않음을 확인했다.

새로운 중간·후반 page를 처음 보이게 한 첫 반복에서는 각각 약 1.1–1.2초 long task가 한 번
관측됐다. 그러나 같은 구간의 selection callback 최대값은 1.8ms 이하였고 이후 5회 warm
반복의 long task는 0회였다. 따라서 이는 selection page-range 경로의 반복 지연이 아니라
새 visible page의 최초 Canvas 렌더 준비 비용으로 분리한다. #2215의 warm drag 완료 조건에는
영향이 없지만 일반 page render 성능 후속에서 참고할 finding이다.

late split에서는 기존 `getCursorRectInCell()`이 p113 fragment를 먼저 반환해
`CursorState`가 pointer `hitTest`의 p114 rect로 복구한다는 경고가 관측됐다. 실제 선택 rect,
highlight, copy 및 성능은 정상이다. page-hinted selection rect 수정과 별개인 cursor-rect
fragment 문제이므로 #2215 범위를 확대하지 않는다.

## 20. cross-page 정확성

p54 `cellPara=1250, offset=0`에서 p55 `cellPara=1275, offset=1`까지 실제 pointer drag했다.
caret/selection rect 경계 자체는 다음 문단과 맞닿으므로 자동화는 `hitTest`가 목표 endpoint를
반환하는 픽셀 영역의 중앙을 사용했다.

| 항목 | HWP | HWPX |
|------|------:|------:|
| endpoint | p54:1250/0 → p55:1275/1 | 동일 |
| visible highlight | 45개 | 45개 |
| copy 길이 | 1,517자 | 1,517자 |
| drag callback p50 / p95 | 1.5 / 2.9ms | 1.7 / 3.0ms |
| rect p95 | 2.4ms | 2.2ms |
| long task | 0회 | 0회 |

copy는 두 형식 모두 `8.3.2.4 거주구역`으로 시작하고 다음으로 끝나 native oracle과
일치했다.

```text
나. 방화문은 어느 쪽에서도 한 사람이 충분히 개폐할 수 있어야 한다.
다
```

## 21. edge auto-scroll

p54에서 container 하단 edge로 실제 pointer를 유지한 뒤 p55 endpoint에서 mouseup했다.

| 항목 | HWP | HWPX |
|------|------:|------:|
| scroll delta | +2,328px | +2,328px |
| 최종 focus | p55 / cellPara 1273 / offset 41 | 동일 |
| visible highlight | 39개 | 39개 |
| copy | 1,351자 | 1,351자, byte 동등 |
| drag callback p50 / p95 | 2.3 / 3.1ms | 2.3 / 3.4ms |
| rect p95 | 2.4ms | 2.7ms |
| long task | 0회 | 0회 |

auto-scroll 중 focus가 p54→p57까지 이동하며 `[54,endPage]` hint가 함께 확장됐고, mouseup
시 p55 선택과 하이라이트가 유지됐다. edge scroll 정책 자체는 변경하지 않았다.

기존 일반 문서 회귀도 다음 결과로 통과했다.

```text
npm run e2e:drag-autoscroll -- --mode=headless
scrollTop 0 → 1529
selection paragraph 0 → 69
highlight 70개
5 assertions passed
```

## 22. selection 중 무효화와 잔여 경계

모든 same-page, exact cross-page, auto-scroll 측정 구간에서 다음은 HWP/HWPX 모두 0회였다.

- `WasmBridge.refreshLayout()`
- `CanvasView.refreshPages()`
- `document-changed` event

따라서 드래그가 pagination 또는 full Canvas refresh를 유발하지 않는다는 Stage 1 판단도 실제
UI 경로에서 유지됐다.

로컬 probe와 raw JSON은 각각 다음에 두었다. CI 계약보다는 이번 성능 자격 검증용으로 작성한
임시 산출물이며 PR에는 포함하지 않는다.

```text
/private/tmp/issue2215_stage3d_probe.mjs
/private/tmp/issue2215_stage3d_results.json
```

## 23. Stage 3-D 판정과 다음 승인 단계

#2215의 구현 목표는 충족했다.

- same-page warm callback p95 50ms 미만: 최대 1.2ms
- exact cross-page/auto-scroll callback p95 50ms 미만: 최대 3.4ms
- warm 반복 long task: 0회
- HWP/HWPX rect·copy oracle: 유지
- mouseup 선택·visible highlight·auto-scroll: 유지
- pagination 및 Canvas full refresh: 0회

다음 단계는 Stage 4 PR 준비다. 최신 upstream rebase, 전체 변경 diff 감사, 승인된 전체 회귀
실행, 이슈 코멘트용 전후 결과 요약과 PR 본문 준비를 수행한다. 최초 visible page cold render와
cursor-rect split fragment finding은 #2215 완료를 막지 않고 후속 후보로 분리한다.

## 24. Stage 3-D 수동 검증 정정

작업지시자 수동 검증에서 UI 113→114는 정상이나 UI 114→115의 동일 cell paragraph 경계를
드래그하면 highlight가 페이지 밖으로 뻗고 긴 지연 뒤 잘못된 endpoint에 선택되는 현상이
확인됐다. 내부 page index로는 113→114이며 UI page는 1-based임을 구분한다.

native probe는 다음 대표 split 경계를 HWP/HWPX에서 동일하게 재현했다.

| UI 경계 | cell paragraph / offset | 반환 page | 최대 page overflow |
|------|------|------|------:|
| 1→2 | 17 / 162..170 | 1쪽만 | +393.4px |
| 56→57 | 1277 / 74..82 | 56쪽만 | +350.6px |
| 114→115 | 2499 / 110..118 | 114쪽만 | +340.3px |

UI 114→115 문단 전체 범위는 115장 full fallback을 거쳐 26.99초가 걸렸다. 따라서 위의
“#2215 구현 목표 충족”과 “후속 후보 분리” 판정은 수동 검증 전 잠정 판정으로 정정한다.
Stage 4로 진행하지 않고 Stage 3-E에서 동일 page cursor pair를 일반 규칙으로 보정한다.

## 25. Stage 3-E 원인 제거

`get_selection_rects_native()`의 기존 cursor 탐색은 한 line segment의 leading과 trailing을
각각 전체 page 후보에서 첫-hit로 찾았다. split paragraph 경계 offset은 이전 page의 trailing과
다음 page의 leading 양쪽에서 유효하므로, 독립 탐색 결과가 서로 다른 page 좌표계에 속할 수
있었다.

Stage 3-E는 각 후보 `PageRenderTree` 안에서 leading/trailing을 함께 해소한 cursor pair만
채택한다. 줄바꿈 문자와 body line-end fallback도 같은 page tree에 한정하고, 다음 segment가
이미 채택한 page보다 이전 fragment로 돌아가지 않게 page 순서를 단조롭게 유지한다. LineSeg,
pagination, Canvas clipping 및 Studio page hint 계약은 변경하지 않았다.

## 26. native 회귀와 성능

HWP/HWPX 각각에서 UI 1→2, 56→57, 114→115의 같은 cell paragraph 경계를 회귀로 추가했다.
모든 경우 endpoint 양쪽 page의 rect가 반환되고 각 rect의 `x + width`가 해당 page 폭 안에
있는지, HWP/HWPX rect와 copy bytes가 같은지 고정했다.

```text
CARGO_TARGET_DIR=/private/tmp/rhwp-task2215-test-target \
  cargo test --offline --test issue_2215_selection_page_range -- \
  --nocapture --test-threads=1

4 passed; 0 failed
```

기존 `issue_658_text_selection_rects`도 `2 passed; 0 failed`로 유지됐다. HWP cpi2499의
114→115 경계 native probe 결과는 다음과 같다.

| 범위 | cold | warm | rect page |
|------|-----:|-----:|------|
| 0..368 | 571.3ms | 2.56ms | 113, 114 |
| 110..118 | — | 0.52–0.59ms | 113, 114 |
| 110..160 | — | 약 0.81ms | 113, 114 |
| 110..200 | — | 약 1.12ms | 113, 114 |
| 110..368 | — | 약 2.4ms | 113, 114 |

수정 전 같은 문단 전체 범위의 115장 fallback은 26.99초였으며, 수정 뒤 같은 범위는 cold
571.3ms, warm 2.56ms로 복구됐다.

## 27. UI 114→115 실제 pointer 재검증

최신 WASM을 다시 빌드하고 사용자 스크린샷과 같은 `(p)` 문단 중간
`cellPara=2499, offset=40`에서 다음 page 첫 줄 `offset=118`까지 실제
`mouse.down → 16 mouse.move → mouse.up`을 수행했다.

| 항목 | HWP | HWPX |
|------|-----:|------:|
| endpoint | p113:2499/40 → p114:2499/118 | 동일 |
| visible highlight | 3개 | 3개 |
| drag callback p50 / p95 | 0.6 / 0.8ms | 0.6 / 0.8ms |
| rect call p95 | 0.3ms | 0.3ms |
| warm long task | 0회 | 0회 |
| hint 전이 | `[113,113]` → `[113,114]` | 동일 |

두 형식의 copy 결과도 다음 문자열로 byte 동등했다.

```text
으로 이루어져야 한다. 수면비행선박에 해상탈출설비(MES)가 설치되어 있는 경우, 시험 시작 시 비상탈출구가 개방상태에 있고 해상탈출설비가
```

위 표시 문자열 뒤에는 공백 1개가 있으며 HWP/HWPX 모두 동일하다.

mouse up 뒤 selection과 양쪽 page highlight가 유지됐으며 page 밖으로 뻗는 rect는 없었다.
계측 probe는 `/private/tmp/issue2215_stage3e_probe.mjs`에 두었고, 환경별 wall-clock을 CI
계약으로 만들지 않기 위해 source에는 포함하지 않는다. 결정적 정확성 계약은 Rust 회귀가
담당한다.

## 28. Stage 3-E 판정

수동 검증에서 확인된 blocker는 동일 page cursor pair 규칙으로 해소됐다.

- UI 1→2, 56→57, 114→115 대표 split 경계: HWP/HWPX GREEN
- 모든 rect: 자기 page 폭 안에 위치
- 사용자 재현과 같은 UI 114→115 실제 pointer drag: endpoint/highlight/copy 유지
- warm drag callback p95: 0.8ms, long task 0회
- 115장 full fallback: 제거
- 기존 same-page, 서로 다른 문단 cross-page, stale/missing hint, #658: 유지

다음 단계는 Stage 4의 전체 CI 성격 검증과 최종 보고서·PR 준비다. 수행계획에 따라 focused
결과를 먼저 공유하고 작업지시자의 별도 승인을 받은 뒤 `cargo test --verbose`와
`cargo clippy --all-targets -- -D warnings`를 실행한다.
