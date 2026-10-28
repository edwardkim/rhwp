# Task M100 #3137 Stage 1 완료보고서 — 거대 셀 입력 전용 성능 하네스

## 1. 완료 범위

최신 `upstream/devel@7995786bf466bb9ce444ca2b31a1d933d13ec61b`에서
`codex/issue-3137-perf-harness` 브랜치를 만들고, #2214 correctness E2E와 분리된
#3137 전용 성능 진단 하네스를 추가했다.

| 파일 | 역할 |
| --- | --- |
| `rhwp-studio/e2e/probe-input-perf-issue3137.mjs` | HWP/HWPX × 영문/숫자/IME × 4개 cadence 계측 |
| `rhwp-studio/package.json` | `e2e:issue-3137-perf` 실행 배선 |
| `rhwp-studio/e2e/MANIFEST.md` | 진단·active·비-CI 성능 하네스 등록 |
| `mydocs/plans/task_m100_3137.md` | Stage 1 하네스와 Stage 2 Rust 내부 계측 계획 |

기존 `issue-2214-page-local-repaint.test.mjs`는 변경하지 않았다. 시간 수치는 머신 의존 진단값으로
기록하고, 문서·cursor·pagination 정합만 hard assertion으로 유지했다.

## 2. 하네스 계약

기본 실행은 다음 24개 시나리오를 한 개의 새 headless Chrome 임시 프로필에서 수행한다.

- HWP/HWPX
- 영문 `a`, 숫자 `1`, 한글 IME `ㅎ → 하 → 한`
- 요청 cadence 0/80/150/250ms
- 시나리오별 warm-up 2회, 논리 입력 20회

각 input sample은 deferred mutation, exact cursor query, 전체 operation, input-to-1/2-rAF,
실제 start interval과 Long Tasks API entry를 수집한다. 일반 입력의 전체 operation은
`InputHandler.executeOperation`, raw mutation 경로인 IME는 동기 `input` dispatch 전체 시간이다.
IME의 history-only `executeOperation(record)`은 별도 보조 지표로 남긴다.

JSON 원시 event/sample과 전체 summary JSON/TSV를
`output/poc/task3137/stage1-current-develop/`에 생성한다. 산출물은 ignored이며 다음 fingerprint를
summary에 기록했다.

| 항목 | 값 |
| --- | --- |
| Chrome | `150.0.7871.182`, 새 headless 임시 프로필 |
| Node | `v24.15.0` |
| production WASM | 7,214,640 bytes |
| WASM SHA-256 | `9225879ea4c533725ee473092597726fe0d108ef75982e4ac761c5a7d78e4b13` |
| fixture | 저장소의 `issue1949_giant_cell_nested_tables_perf.{hwp,hwpx}` |

## 3. production WASM freshness 확인

첫 smoke는 2026-07-26에 만들어진 기존 `pkg/rhwp_bg.wasm`
(24,788,644 bytes, SHA-256 `e3a9879f...`)을 사용해 cursor query 약 440ms가 나왔다. 이는 최신
checkout에서 새로 만든 production 산출물이 아니므로 기준선에서 폐기했다.

`wasm-pack build --target web --out-dir pkg`의 release profile과 `wasm-opt`를 완주한 뒤 같은 smoke는
operation p95 44.3ms, cursor p95 43.8ms로 돌아왔다. 하네스가 Git·WASM·fixture·자기 파일
fingerprint를 함께 기록하므로 이후 stale build를 결과에서 구분할 수 있다.

## 4. 전체 매트릭스 결과

아래 셀은 `operation p95 / cursor query p95`(ms)다.

| 포맷·입력 | 0ms | 80ms | 150ms | 250ms |
| --- | ---: | ---: | ---: | ---: |
| HWP 영문 | 43.1 / 42.7 | 44.1 / 43.7 | 54.4 / 53.8 | 66.6 / 65.7 |
| HWP 숫자 | 44.2 / 43.8 | 44.1 / 43.6 | 53.6 / 53.0 | 66.1 / 65.3 |
| HWP IME | 45.1 / 44.5 | 46.6 / 45.8 | 53.5 / 52.6 | 65.7 / 64.5 |
| HWPX 영문 | 46.1 / 45.6 | 46.1 / 45.5 | 53.7 / 53.0 | 63.7 / 62.9 |
| HWPX 숫자 | 46.4 / 45.8 | 46.4 / 45.9 | 54.1 / 53.4 | 65.9 / 65.1 |
| HWPX IME | 46.9 / 46.3 | 47.0 / 46.2 | 54.1 / 53.1 | 66.4 / 65.1 |

input-to-2-rAF p95 범위는 다음과 같다.

| 요청 cadence | p95 범위 | 해석 |
| ---: | ---: | --- |
| 0ms | 103.8~110.8ms | 다음 input task가 앞선 sample의 2-rAF까지 겹치는 연속 입력 |
| 80ms | 61.6~65.5ms | 다음 입력 전에 2-rAF가 대체로 완료 |
| 150ms | 71.1~72.7ms | cursor query가 50ms를 넘어 Long Task로 분류되기 시작 |
| 250ms | 82.8~86.1ms | cursor query p95가 약 63~66ms까지 증가 |

전체 800 event sample은 모두 stable mutation이었다.

| 계약 | 결과 |
| --- | ---: |
| stable sample | 800 / 800 |
| deferred insert / replace | 480 / 320 |
| exact path-near cursor query | 800 |
| direct `getCursorRectInCell` | 0 |
| 동기 WASM/input flush | 0 / 0 |
| resumable begin / step | 0 / 0 |
| browser page error | 0 |
| frame budget 충족 scenario | 0 / 24 |

## 5. cadence 대조

기본 실행의 0→80→150→250ms 순서가 결과를 만들었는지 배제하기 위해 HWP 영문을
250→150→80→0ms 역순으로 다시 실행했다.

| cadence | operation p95 | cursor p95 | flush |
| ---: | ---: | ---: | ---: |
| 250ms | 65.4ms | 64.4ms | 0 |
| 150ms | 53.1ms | 52.3ms | 0 |
| 80ms | 46.6ms | 46.0ms | 0 |
| 0ms | 46.3ms | 45.8ms | 0 |

역순에서도 관계가 유지됐다. cadence별로 wrapper가 관측한 호출 종류와 횟수도 동일했다.
따라서 이 현상은 단순 실행 순서나 #3412의 idle 전체 pagination flush 재발은 아니다.

현재 증거로 확정할 수 있는 범위는 다음과 같다.

- 영문·숫자·IME 차이보다 공통 exact cursor query가 지배적이다.
- mutation p95는 약 0.2~0.6ms이며 operation과 cursor의 차이는 대체로 1ms 안팎이다.
- 긴 idle 뒤 exact query 자체가 더 느려진다.
- CPU power/cache warmness와 page-tree rebuild 내부 하위 비용 가운데 무엇이 원인인지는 아직
  분리되지 않았으므로 Stage 2 Rust 계측으로 확정해야 한다.

## 6. 회귀 검증

| 명령 | 결과 |
| --- | --- |
| `node --check e2e/probe-input-perf-issue3137.mjs` | 통과 |
| 기본 24개 성능 매트릭스 | 24/24 통과 |
| HWP 영문 cadence 역순 대조 | 4/4 통과 |
| `npm test` | 670 passed / 0 failed |
| `npm run build` | production build 통과 |
| `npm run e2e:issue-2214 -- --formats=hwp --runs=1` | focused·raw·삭제·IME·저장·인쇄 전 단계 통과 |

#2214 HWP focused 결과는 `flush=0`, `begin=1`, `step=115`, stable operation p95 47.2ms,
boundary operation 80.1ms였다. 저장은 flush 뒤 HWP export, 인쇄는 flush 뒤 115쪽 렌더를 유지했다.

## 7. 다음 단계

Stage 2에서는 제품 동작을 바꾸기 전에 Rust cursor query 내부를 다음 구간으로 나눠 계측한다.

1. `find_pages_for_paragraph`
2. page-tree cache hit/miss
3. `build_page_tree_cached`
4. tree traversal
5. `compute_char_positions`

특히 동일 query의 80ms 대비 150/250ms cadence 증가가 어느 구간에서 발생하는지 함께 기록한다.
지배 구간이 확정된 뒤에만 edit revision 기반 focused cell/line geometry 재사용 구현으로 넘어간다.
