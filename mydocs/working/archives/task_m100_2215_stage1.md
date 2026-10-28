# 단계별 완료 보고서 — Task M100 #2215 Stage 1

## 1. 결론

#2215의 지배 병목은 확정됐다. 셀 내부의 짧은 same-page 선택도 외부 표 host 문단이 걸친
115쪽을 전부 후보로 열거하고, 매 호출마다 115개 page tree를 uncached로 다시 구축한다.
실제 선택 rect는 한 페이지의 1개뿐이지만 HWP/HWPX 모두 한 번의 WASM selection rect
호출이 약 1.8초 걸렸고, 같은 문서에서 다섯 번 반복해도 개선되지 않았다.

따라서 rAF throttle은 이 문제를 완화할 수 없다. 한 callback 자체가 약 108 프레임
(60Hz 기준)을 차단한다. page range 제한과 shared cached tree 사용을 함께 적용해야 하며,
둘 중 하나만 적용하는 안은 완료안이 될 수 없다.

## 2. 측정 환경

- 날짜: 2026-07-19
- 기준 소스: `upstream/devel@eb9c7f1f`
- 아키텍처: Apple Silicon `arm64`
- Node.js: `v24.15.0`
- WASM: 로컬 `pkg/rhwp_bg.wasm` (2026-07-17 02:06, 6,959,573 bytes)
- 현재 기준 소스와 비교한 관련 경로:
  - `src/document_core/queries/cursor_nav.rs`
  - `src/wasm_api.rs`
  - `rhwp-studio/src/engine/input-handler.ts`
  - `rhwp-studio/src/engine/input-handler-mouse.ts`
- 위 관련 경로는 측정 worktree와 `upstream/devel` 사이 diff가 없다.
- 일회성 probe 위치: `/private/tmp/rhwp-task2215-probe/`
- probe와 산출물은 저장소 및 PR에 포함하지 않는다.

## 3. 코드 경로 재확인

```text
mousemove requestAnimationFrame
→ updateTextSelectionDragFromPointer
→ updateCaretDuringDrag
→ updateSelection
→ getSelectionRectsInCell
→ get_selection_rects_native
```

`get_selection_rects_native()`의 현재 셀 선택 경로는 다음 순서다.

1. `lookup_para`를 실제 cell paragraph가 아닌 외부 표 `parentParaIndex`로 정한다.
2. `find_pages_for_paragraph(section, lookup_para)`로 115쪽 전체를 얻는다.
3. “최대 2페이지”라는 주석과 달리 모든 후보에서 `build_page_tree()`를 호출한다.
4. 함수 로컬 `tree_cache`는 다음 selection 호출에서 재사용되지 않는다.
5. 이미 전 페이지를 구축한 뒤 실제 endpoint를 포함하는 tree를 순차 검색한다.

직접 Node→WASM API를 호출했으므로 pointer hit test, DOM overlay, Canvas repaint와
pagination을 제외한 selection rect 자체의 비용을 격리했다.

## 4. HWP/HWPX same-page 기준선

셀 경로는 `section=0, parentPara=0, control=2, cell=2`다. 첫 페이지, 중간,
후반의 실제 문단을 각각 선택했다.

| 형식 | 셀 문단 | 실제 페이지 | 선택 범위 | 반환 rect | selection 호출 반복 중앙값 |
|------|---------:|------------:|-----------|----------:|----------------------------:|
| HWP | 5 | 0 | 0..10 | 1개 / p0 | 1,817ms |
| HWP | 1250 | 54 | 0..1 | 1개 / p54 | 1,824ms |
| HWP | 2499 | 113 | 0..1 | 1개 / p113 | 1,827ms |
| HWPX | 5 | 0 | 0..10 | 1개 / p0 | 1,809ms |
| HWPX | 1250 | 54 | 0..1 | 1개 / p54 | 1,811ms |
| HWPX | 2499 | 113 | 0..1 | 1개 / p113 | 1,826ms |

별도 동일 범위 5회 반복에서도 HWP는 1,791–1,844ms, HWPX는
1,772–1,812ms였다. 첫 호출 뒤에도 비용이 유지되므로 layout warm-up이나 함수 로컬
tree 재사용으로 해결되지 않는다.

## 5. cursor 조회와의 대조

동일 문단의 `getCursorRectInCell`은 endpoint를 찾을 때까지만 페이지를 순차 탐색한다.

| 형식 | p0 | p54 | p113 |
|------|---:|----:|-----:|
| HWP | 55ms | 880ms | 1,812ms |
| HWPX | 53ms | 867ms | 1,803ms |

페이지 위치에 거의 선형으로 증가한다. 반면 selection rect는 p0에서도 약 1.8초다.
이는 selection 경로가 endpoint를 찾기 전에 115쪽 전체 tree를 선구축한다는 코드 분석과
일치한다.

## 6. cross-page 기준선

셀 문단 1250의 시작부터 1275의 offset 1까지 선택했다.

| 형식 | 실제 rect 페이지 | rect 수 | 호출 시간 |
|------|------------------|--------:|----------:|
| HWP | 54–55 | 45 | 2,626ms |
| HWPX | 54–55 | 45 | 1,981ms |

실제 결과에는 두 페이지만 필요하다. endpoint 사이의 명시적 page range가 전체 115쪽
host 범위보다 정확한 후보라는 근거를 확보했다. HWP 첫 측정의 추가 비용은 cold 상태가
포함된 값이므로 형식 간 성능 우열로 해석하지 않는다.

## 7. 정확성 기준선

기존 selection rect 정확성 회귀를 현재 소스에서 별도 target으로 실행했다.

```text
cargo test --test issue_658_text_selection_rects
2 passed; 0 failed; finished in 0.10s
```

따라서 #658의 줄 시작 및 페이지 폭 기하 계약은 현재 GREEN이다. Stage 2와 구현 이후에도
이 결과를 유지해야 한다.

## 8. Stage 1 판정

| 항목 | 판정 |
|------|------|
| 115쪽 후보 열거 | 지배 원인 확정 |
| uncached `build_page_tree()` 반복 | 지배 원인 확정 |
| cold-only 비용 | 기각 — warm 반복에서도 약 1.8초 |
| 반환 rect 양 자체 | 기각 — same-page는 1개뿐 |
| HWP 전용 문제 | 기각 — HWPX도 동형 |
| pagination/Canvas refresh | 직접 원인 아님 — 직접 WASM 호출로도 재현 |
| rAF throttle 단독 보완 | 기각 — callback 내부 호출이 약 1.8초 |

## 9. Stage 2 권고

1. 기존 전체 탐색의 rect JSON과 copy 문자열을 oracle로 고정한다.
2. same-page는 endpoint page 한 장을 1차 후보로 사용하고, 페이지 fragment 경계에서만
   인접 페이지가 필요한지 통제한다.
3. cross-page는 두 endpoint 사이의 명시적 범위만 후보로 사용한다.
4. 제한한 후보에서 `build_page_tree_cached()`를 사용한다.
5. hint 누락·범위 밖·endpoint 미발견 시 정확한 fallback을 유지하되, 정상 Studio drag가
   fallback으로 빠지지 않는 회귀를 추가한다.
6. 기존 positional API 호환성을 유지할 options/new API 형태는 Stage 2 결과 뒤
   구현계획서에서 확정한다.

## 10. 남은 UI 검증

최신 devel Studio 서버는 정상 기동했으나 이 세션에는 앱 내 브라우저 제어 런타임이
노출되지 않아 실제 pointer drag 자동 계측은 수행하지 않았다. 서버는 종료했다.

다만 직접 WASM 호출만으로 한 번의 `getSelectionRectsInCell`이 약 1.8초 걸리므로 실제
`updateSelection()`과 selection rAF callback의 하한도 같은 수준이다. 지배 원인 판정에는
영향이 없다. 실제 focus offset 전진, visible highlight, mouseup 유지와 callback p95는
Stage 3의 Studio 회귀 설계 및 구현 후 E2E 게이트로 유지한다.
