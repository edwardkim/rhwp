# Task M100 #3137 Stage 3 완료보고서 — focused cell cursor geometry 재사용

## 1. 결론

거대 표 셀의 연속 stable 입력에서 mutation 직후 수행하던 exact cursor query와
`build_page_tree(0)` 전체 재구축을 동기 입력 경로에서 제거했다.

최종 production WASM과 새 headless Chrome으로 실행한 24개 시나리오, 800개 측정 sample에서
다음 결과를 확인했다.

| 항목 | 결과 |
| --- | ---: |
| focused geometry 제공 / 적용 | 800 / 800 |
| exact cursor query | 0 |
| sync flush / begin / step | 0 / 0 / 0 |
| stable operation p95 범위 | 0.7–2.3ms |
| mutation p95 범위 | 0.4–1.0ms |
| cursor update p95 범위 | 0.0ms |
| frame-budget gate | 24 / 24 통과 |

Stage 2의 stable exact query p95 44–66ms가 측정 구간에서 0회가 됐으므로, cursor hot path의
page-tree rebuild 병목은 제거됐다. 다만 첫 입력, flow 경계, pagination commit 뒤에는 의도적으로
exact query를 사용한다. 또한 입력 뒤 첫 page repaint는 여전히 page tree를 다시 만들기 때문에
`input → 2-rAF` p95 67.8–85.2ms의 별도 long task가 남는다. 따라서 Stage 3는 동기
caret/operation 병목을 해결했지만, 사용자가 보는 페이지 반영 지연까지 모두 해결한 최종 단계는 아니다.

## 2. 구현

### 2.1 Rust mutation 결과

deferred flat-cell insert/replace/delete가 편집 전후의 문단 로컬 caret x를 계산하고, 다음 조건을 모두
만족할 때만 mutation JSON에 `focusedCursorGeometry`를 넣는다.

- 편집 전후 visual line index와 line start가 동일
- 원본 `LineSeg`의 위치·높이·폭·tag 서명이 동일
- 셀 flow advance가 동일하고 이전 revision에도 pending flow change가 없음
- BMP text이며 inline control, 줄바꿈, 탭, PUA display 확장, 글자겹침, 각주 marker가 없음
- 지원하는 language run이며 left 또는 마지막 justify line
- 계산한 x delta가 유한값

payload는 `baseRevision`, `revision`, `sourceCharOffset`, `targetCharOffset`, `deltaX`를 포함한다.
mutation 내부에서 기존 exact cursor API를 다시 호출하지 않으므로 비용을 다른 위치로 옮기지 않는다.

### 2.2 Studio 적용

WASM bridge와 command effect가 payload를 typed geometry로 전달한다. `CursorState`는 직전
exact/hit rect의 absolute page/cell 원점을 유지하고, 다음 `moveTo` 한 번에 local `deltaX`만 적용한다.

적용 전에 다음을 다시 검증한다.

- 현재 source와 payload source의 section, paragraph, cell path, char offset 일치
- 직전 fast-path revision과 `baseRevision` 일치
- horizontal cell
- 직전 rect가 overflow되지 않았고 결과 x가 cell bounds 안
- mutation target과 실제 이동 target 일치

revision 또는 target이 다르거나 geometry가 없는 mutation, flow 경계, 동기 flush,
shadow pagination commit, vertical cell에서는 상태를 무효화하고 기존 exact query로 복구한다.
여러 mutation effect가 한 command에 합쳐져 중간 rect를 보장할 수 없는 경우에도 geometry를 버린다.

## 3. 첫 입력과 fallback

fixture의 저장된 원본 `LineSeg`는 첫 local reflow에서 합성 metric으로 정규화된다. 이때 편집 전후
line 서명이 달라지므로 첫 입력은 geometry를 만들지 않고 exact query를 한 번 수행한다. 그 exact rect가
이후 revision chain의 기준점이 된다.

#2214 연속 입력 trace에서 포맷별 cursor query는 총 3회였다.

1. 첫 입력의 line-metric 정규화
2. 56번째 flow 경계
3. shadow pagination commit 뒤 exact rect 재동기화

HWP multi-update IME는 `46.5 / 1.0 / 0.8ms`, HWPX는 `45.2 / 0.7 / 0.7ms`였다. 첫 입력까지
fast path로 만들려면 저장 metric과 합성 metric의 동등성을 별도로 증명하거나 mutation이 absolute
origin까지 반환해야 한다. Stage 3에서는 잘못된 caret을 허용하는 완화보다 보수적 fallback을 유지했다.

## 4. 실행 환경

| 항목 | 값 |
| --- | --- |
| 브랜치 | `codex/issue-3137-perf-harness` |
| working-tree HEAD | `4d547d56835d33cc17b0fee92cbfdb379e763520` + Stage 3 변경 |
| 기준 `upstream/devel` | `6c18949bb39c1fa5026b556ef7308ae99cdcf489` |
| Chrome | `150.0.7871.187`, 새 headless 임시 프로필 |
| Node | `v24.15.0` |
| production WASM | 7,250,961 bytes |
| WASM SHA-256 | `8a3bc1720363b5e50c80038275f117fb2fa8646261ab1778a2dd33719bfaf2ca` |
| 성능 결과 | `output/poc/task3137/stage3-final-full-matrix/` |
| correctness 결과 | `output/poc/task3137/stage3-final-issue2214/` |

## 5. 최종 성능 행렬

아래 값은 `stable operation p95`(ms)다. 각 셀의 측정 sample은 20개이고, IME sample은
`ㅎ → 하 → 한` 세 mutation을 포함한다.

| 포맷·입력 | 0ms | 80ms | 150ms | 250ms |
| --- | ---: | ---: | ---: | ---: |
| HWP 영문 | 0.7 | 0.8 | 1.1 | 1.7 |
| HWP 숫자 | 0.7 | 0.8 | 1.0 | 1.9 |
| HWP IME | 0.9 | 1.1 | 1.3 | 2.3 |
| HWPX 영문 | 0.7 | 0.8 | 1.1 | 1.8 |
| HWPX 숫자 | 0.7 | 0.8 | 0.9 | 1.6 |
| HWPX IME | 0.9 | 1.1 | 1.4 | 2.1 |

800개 측정 sample 모두 cursor target, 최종 text, 115쪽, deferred mutation 계약을 만족했다.
geometry 제공·준비는 800/800, exact cursor query와 동기 pagination 작업은 모두 0회였다.

## 6. #2214/#2424 correctness 게이트

최종 WASM과 TypeScript 상태에서 HWP/HWPX를 각각 1회 실행했다.

| 항목 | HWP | HWPX |
| --- | ---: | ---: |
| stable operation p95 | 0.8ms | 0.6ms |
| flow boundary | 56 | 56 |
| boundary operation | 75.9ms | 76.0ms |
| begin / steps / flush | 1 / 115 / 0 | 1 / 115 / 0 |
| Backspace / Delete WASM | 2.0 / 1.6ms | 1.6 / 1.4ms |
| raw stable/boundary IME·iOS | GREEN | GREEN |
| save barrier | HWP 229,376 bytes | HWPX 225,699 bytes |
| print barrier | 115쪽 | HWP suite에서 통과 |

flow 경계 전후 visual crop과 pagination 완료까지의 crop은 기존 #2214 비교 계약을 통과했다.
저장과 인쇄는 pending pagination을 먼저 flush한 뒤 export/render하는 순서를 유지했다.

## 7. 테스트

| 검증 | 결과 |
| --- | --- |
| Rust focused geometry와 exact rect HWP/HWPX 대조 | 통과 |
| Rust `cargo test --lib` | 2,983 passed / 0 failed / 7 ignored |
| Rust `cargo clippy --lib -- -D warnings` | 통과 |
| Rust `cargo fmt --all --check` | 통과 |
| Studio `npm test` | 677 passed / 0 failed |
| focused revision/fallback 행위 테스트 | 2 passed / 0 failed |
| Studio `npm run build` | 통과 |
| Stage 3 전체 성능 행렬 | 24 / 24, 800 / 800 |
| #2214 HWP/HWPX focused/raw/delete/IME/save/print | 전 단계 통과 |

## 8. 남은 작업

다음 Stage 4는 cursor query가 아니라 `document-page-invalidated` 뒤 첫 animation frame에서 수행하는
page repaint를 대상으로 한다. 현재 fast caret 적용 뒤에도 `refreshInvalidatedPageNow`가 page tree를
다시 만들며 67.8–85.2ms의 `input → 2-rAF` p95와 long task를 만든다.

우선순위는 다음과 같다.

1. stable same-flow edit에서 focused page의 local render patch 또는 reusable page layer 범위를 계측한다.
2. cursor용 geometry와 화면 repaint용 invalidation을 분리해 전체 page-tree rebuild를 피한다.
3. flow 경계와 shadow pagination commit은 기존 full repaint를 유지한다.
4. 첫 입력 line-metric 정규화의 안전한 origin 재사용 가능성을 별도 검증한다.

exact query를 단순히 다음 frame으로 옮기는 방식은 Stage 4 해결책이 아니다. 이미 남은 지연이
page repaint frame에 있으므로, rebuild 자체의 범위를 줄여야 한다.
