# 단계별 완료 보고서 — Task M100 #2400 Stage 2

## 1. 결론

다중 페이지 표의 pointer hit 경로가 현재 page의 표 fragment bbox만 사용하도록 수정했다.
UI 114쪽 `어 있는 경우` 첫 글자 앞 클릭은 HWP/HWPX 모두 표 객체 선택 없이
`cellParaIndex=2499`, `charOffset=77`에 캐럿이 진입한다. 같은 page fragment의 실제 하단
테두리 클릭은 계속 표 객체를 선택하므로 기존 ±5px 외곽 선택 계약도 보존된다.

수정은 table bbox 조회와 Studio hit 판정에 한정된다. pagination, line break, 폰트 metric,
selection rect semantic은 변경하지 않았다.

## 2. 구현

### native/WASM

- legacy `get_table_bbox_native()` / `getTableBBox()`의 첫 fragment 계약은 유지했다.
- `get_table_bbox_at_page_native()` / `getTableBBoxAtPage()`를 추가했다.
- 새 API는 지정 page tree 한 장만 조회하며 다른 page fragment로 fallback하지 않는다.
- 잘못된 표 참조, 범위 밖 page, 지정 page에 fragment가 없는 경우 오류를 반환한다.

### Studio

- `WasmBridge.getTableBBoxAtPage()` typed wrapper를 추가했다.
- 일반 셀 클릭, 선택된 표의 셀 재진입·이동 시작, 선택된 표 hover의 세 경로가 pointer
  `pageIdx`를 전달하도록 변경했다.
- 표 외부 탐색의 page layout 우선 경로는 유지하고 fallback에도 같은 page를 전달했다.
- ±5px 외곽 판정을 `isPointNearBoxBorder()` pure helper로 분리했다.

## 3. source 회귀

권위 HWP/HWPX 115쪽 샘플에서 다음 계약을 고정했다.

| 항목 | 결과 |
| --- | --- |
| legacy bbox | `pageIndex=0`, 기존 계약 유지 |
| page-scoped bbox | `pageIndex=113`, 현재 fragment 반환 |
| 재현점 `(142.8, 1057.3)` | legacy bbox에서는 border, 현재 bbox에서는 text |
| 범위 밖 `pageIndex=115` | 오류, 첫 fragment fallback 없음 |

Studio 테스트는 확정 좌표와 좌·우·상·하 ±5px 계약, 세 page 전달 경로를 검증한다.

## 4. 실제 pointer 검증

로컬 개발 WASM과 headless Chrome에서 HWP/HWPX를 각각 검증했다.

### #2400 직접 시나리오

| 입력 | HWP | HWPX |
| --- | --- | --- |
| UI 114쪽 offset 77 텍스트 클릭 | 표 선택 없음, offset 77 캐럿 | 표 선택 없음, offset 77 캐럿 |
| page 113 실제 fragment 하단 클릭 | 표 `sec=0, ppi=0, ci=2` 선택 | 동일 |

첫 fragment 하단과 클릭점 거리는 3.7px이지만 현재 fragment 하단과는 18.6px이다.
수정 후 native 경계 판정은 `false`였고 실제 pointer 결과도 일치했다.

### 인접 fragment 텍스트 클릭

HWP/HWPX 각각 다음 여섯 위치를 실제 pointer로 클릭했다. 12건 모두 표 객체 선택 없이
요청한 page·문단·offset에 캐럿이 진입했다.

| UI page 경계 | 앞 fragment | 뒤 fragment |
| --- | --- | --- |
| 1→2 | page 0 / para 17 / offset 162 | page 1 / para 17 / offset 170 |
| 56→57 | page 55 / para 1277 / offset 74 | page 56 / para 1277 / offset 82 |
| 114→115 | page 113 / para 2499 / offset 110 | page 114 / para 2499 / offset 118 |

### #2215 drag 회귀

UI 114→115 drag를 HWP/HWPX에서 실제 pointer로 다시 수행했다.

- 시작: page 113 / para 2499 / offset 40
- 끝: page 114 / para 2499 / offset 118
- 두 포맷 모두 selection 유지, highlight 3개, 복사 문자열 동일
- warm drag handler: HWP p95 5.0ms, HWPX p95 4.9ms
- selection rect: 두 포맷 p95 약 1.2ms

## 5. focused 검증 결과

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 통과 |
| 신규 native #2400 HWP/HWPX 회귀 | 1 passed |
| legacy `test_get_table_bbox` | 1 passed |
| `issue_2215_selection_page_range` | 4 passed |
| `issue_717_table_cell_hit_test` | 4 passed |
| `issue_nested_table_border` | 2 passed |
| `issue_919_textbox_hit_test` | 5 passed |
| `npm --prefix rhwp-studio test` | 418 passed (최종 rebase 후 449 passed) |
| `npm --prefix rhwp-studio run build` | 통과 |
| `git diff --check` | 통과 |

개발용 `wasm-pack build --dev --target web --out-dir pkg`도 통과했다. 실제 포인터 probe와
스크린샷은 일회성 검증 산출물이므로 `/private/tmp`에만 유지하고 PR에는 포함하지 않는다.

## 6. 전체 CI

focused 결과 공유와 별도 승인 뒤 PR 전 전체 CI를 실행했다.

```text
cargo test --verbose                         통과
cargo clippy --all-targets -- -D warnings  통과
```

`cargo test --verbose`는 library 2,288 passed / 0 failed / 7 ignored를 포함해 전체 integration,
round-trip, visual baseline, doc-test를 완료했다. #1949 전체 115쪽 렌더, #2185 단일 문자 편집,
#2214 cache coherence, #2215 page-hinted selection 등 장시간 회귀도 모두 통과했다.

다음 단계는 변경 요약과 실제 pointer 근거를 #2400에 공유하고 push/PR 생성 승인을 받는
것이다.

## 7. 최신 devel rebase 검증

PR 직전 `upstream/devel@3eac4ae0`으로 3개 작업 커밋을 충돌 없이 rebase했다. 유입된 17개
커밋은 주로 CanvasKit 렌더링 강화와 CFB 헤더 검증이며 page-scoped table hit-test와 의미
충돌은 없었다.

최종 rebase 후 다음 focused 검증을 다시 통과했다.

- 신규 #2400 HWP/HWPX native 회귀 1 passed
- Studio 449 passed, production build 통과
- #2215 selection page range 4 passed
- #717 table cell hit-test 4 passed
- #919 textbox hit-test 5 passed

## 8. 작업지시자 수동 검증 후 캐럿 기하 보완

작업지시자가 Draft PR #2425 로컬 서버에서 UI 114쪽 `어 있는 경우` 앞을 클릭했을 때
표 객체 선택은 해소됐지만 클릭 위치에 캐럿이 보이지 않는 잔여 증상을 확인했다. 최신 빌드
미반영이 아니라 pointer hit 이후 같은 page에서 수행되는 경로 기반 캐럿 재조회가 다른
continuation run을 선택한 것이 원인이었다.

| 단계 | page | x | y | 결과 |
| --- | ---: | ---: | ---: | --- |
| `hitTest.cursorRect` | 113 | 150.8 | 1049.3 | `어` 앞의 정확한 위치 |
| 기존 `CursorState.updateRect()` 재조회 | 113 | 670.9 | 1023.7 | 같은 page의 이전 줄 끝 |
| 기존 화면 caret | 113 | 670.9 | 1023.7 | 화면 x 약 924px로 밀림 |

페이지 번호만 비교하던 기존 폴백은 두 결과가 모두 page 113이라 불일치를 감지하지 못했다.
이를 전역 좌표 허용 오차로 보정하지 않고, 방금 수행한 pointer hit 경로에만
`moveToHit()`을 도입했다. hit-test가 `cursorRect`를 제공하면 이를 직접 사용하고 좌표가 없는
호환 경로에서만 기존 경로 재조회를 수행한다. 일반 편집·키보드 이동은 기존 `moveTo()` 재조회를
유지해 오래된 pointer 좌표를 재사용하지 않는다.

작업지시자의 후속 수동 확인에서 클릭 후 약간의 지연도 관찰됐다. 깨끗한 브라우저 세션에서
계측한 결과, 최초 pointer 클릭은 `moveToHit()`의 불필요한 선행 `updateRect()`가 legacy
`getCursorRectByPath()` 전체 탐색을 유발해 약 26.6초를 사용했다. 위 direct-hit 정책으로 이
탐색을 제거했다. 반복 클릭 중앙값 약 252ms 가운데 약 242ms는 별도 `hitTestFootnote()` 경로로
확인되어 #2400의 캐럿 기하 수정과 분리해 성능 후속 #2428로 추적한다.

### 최초 클릭 탐색 제거 후 재계측

같은 HWP의 UI 114쪽 offset 77/78을 12회 번갈아 클릭했다.

| 구간 | 호출 수 | 첫 클릭 | p50 | p95 |
| --- | ---: | ---: | ---: | ---: |
| 전체 `mousedown` 처리 | 12 | 268.6ms | 258.1ms | 268.6ms |
| `moveToHit` | 12 | 0ms | 0ms | 0ms |
| `getCursorRectByPathNear` | 0 | - | - | - |
| legacy `getCursorRectByPath` | 0 | - | - | - |
| `hitTestFootnote` | 12 | 248.7ms | 248.2ms | 248.8ms |

최초 클릭 약 26.6초의 cursor lookup은 0회로 제거됐다. 캐럿은 계속
`pageIndex=113, x=150.8, y=1049.3, height=16`에 진입했다. 남은 warm 지연은 #2428의
각주 page-local fast-negative/prefilter 범위로 분리했다.

### 보완 검증

- `npm --prefix rhwp-studio test`: 452 passed
- `npm --prefix rhwp-studio run build`: 통과
- HWP/HWPX UI 114쪽 offset 77:
  - `tableObjectSelected=false`
  - cursor rect `(150.8, 1049.3)`
  - DOM caret 화면 x 약 404px로 `어` 앞에 표시
  - 같은 fragment 실제 하단 클릭은 표 객체 선택 유지
- #2215 UI 114→115 실제 pointer drag HWP/HWPX:
  - selection 유지, highlight 3개, 복사 문자열 동일
  - HWP drag p95 5.5ms / rect p95 1.4ms
  - HWPX drag p95 4.7ms / rect p95 1.3ms
- HWP/HWPX page 0/1, 55/56, 113/114의 전·중·후반 텍스트 클릭 12건:
  - 표 객체 오인 0건
  - 기대 page/cell paragraph/offset 캐럿 일치 12건
- 후속 성능 이슈: [#2428](https://github.com/edwardkim/rhwp/issues/2428)

일회성 계측 probe와 스크린샷은 `/private/tmp`에만 유지한다. 작업지시자는 같은 7716 서버에서
UI 114쪽 `어` 앞의 시각 캐럿 진입이 정상화됐음을 확인했다. #2400의 잔여 게이트는 이 보완을
Draft PR #2425에 반영하고 CI·리뷰를 받는 것이다.
