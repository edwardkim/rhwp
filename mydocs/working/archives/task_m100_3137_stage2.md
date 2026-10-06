# Task M100 #3137 Stage 2 완료보고서 — cursor query Rust 내부 구간 계측

> 이 문서는 연구 브랜치에서 수행한 일회성 원인 계측의 기록이다.
> 측정 결론은 보존하지만 진단 WASM API, `web-time` 직접 의존성,
> `--cursor-breakdown` 옵션은 정리 PR의 런타임에서 제거한다.

## 1. 결론

거대 표 셀의 stable 입력 지연은 `find_pages_for_paragraph`, 렌더 트리 순회,
`compute_char_positions`가 아니라 **편집 직후 cache miss에서 수행하는
`build_page_tree(0)` 전체 재구축**이 지배한다.

production WASM의 800개 profile에서 확인한 결과는 다음과 같다.

| 항목 | 결과 |
| --- | ---: |
| stable input profile | 800 / 800 |
| page-tree call | 800 |
| cache miss / hit | 800 / 0 |
| fallback query | 0 |
| query가 일치한 페이지 | 모두 page 0 |
| 후보 페이지 수 | 모두 115 |
| `build_page_tree` 누적 비중 | **99.86%** |
| traversal exclusive p95 | 0.0ms |
| `compute_char_positions` p95 | 0.1ms |

입력 cadence가 80ms에서 150/250ms로 길어질 때 증가하는 시간도 전부
`build_page_tree` 안에서 발생한다. #3412/#3462의 idle pagination flush 회귀가 다시 발생한
것이 아니며, 영문·숫자·IME가 공유하는 exact cursor geometry 경로의 별도 병목이다.

## 2. 구현 범위

normal `getCursorRectByPathNear`의 반환·호출 횟수·cache 동작은 유지하고, 같은 query를 한 번만
실행해 좌표와 profile을 함께 반환하는 opt-in diagnostic API를 추가했다.

| 파일 | 역할 |
| --- | --- |
| `src/document_core/queries/cursor_rect.rs` | profile DTO, diagnostic cache builder/walker, native 동등성 테스트 |
| `src/wasm_api.rs` | `getCursorRectByPathNearDiagnostic` WASM export |
| `rhwp-studio/src/core/wasm-bridge.ts` | typed diagnostic bridge |
| `rhwp-studio/e2e/probe-input-perf-issue3137.mjs` | `--cursor-breakdown` 수집·JSON/TSV 요약 |

하네스 기본 실행은 계속 normal API를 사용한다. `--cursor-breakdown`을 지정한 경우에만
`getCursorRectByPathNear` wrapper가 diagnostic API를 호출하고, 반환된 `rect`를 기존 호출자에게
그대로 전달한다. 따라서 profile을 얻기 위해 cursor query를 두 번 실행하지 않는다.

Rust profile은 다음 구간과 부가 상태를 기록한다.

1. path parse와 대상 문단 resolve
2. `find_pages_for_paragraph`와 hint page 정렬
3. page-tree cache hit/miss
4. `build_page_tree`, cache clone/store
5. tree traversal inclusive/exclusive
6. `compute_char_positions`
7. rect format과 나머지 시간

normal hot path의 재귀 walker에는 node별 timing 분기를 넣지 않았다. diagnostic API만 별도 walker를
사용하므로 기본 제품 경로의 동작과 계측 오버헤드를 분리했다.

## 3. 실행 환경

| 항목 | 값 |
| --- | --- |
| 기준 브랜치 | `codex/issue-3137-perf-harness` |
| 계측 당시 HEAD | `871fbbbb1b1211faffcc0c4748b2d5980d986c11` + Stage 2 working tree |
| upstream 기준 | `7995786bf466bb9ce444ca2b31a1d933d13ec61b` |
| Chrome | `150.0.7871.182`, 새 headless 임시 프로필 |
| Node | `v24.15.0` |
| production WASM | 7,228,534 bytes |
| WASM SHA-256 | `b2bdedffbdb20256368f25333c6be269608b30752ed07745e95097e614895d67` |
| 결과 경로 | `output/poc/task3137/stage2-current-develop/` |

첫 browser smoke에서 `std::time::Instant`가 `wasm32-unknown-unknown`에서 지원되지 않아 panic했다.
해당 실행은 입력 1회 뒤 중단됐으므로 폐기했다. WASM에서는 브라우저
`performance.now()` 기반 `web_time::Instant`, native에서는 `std::time::Instant`를 사용하도록
분기한 뒤 production WASM을 다시 빌드하고 모든 수치를 새로 측정했다.

## 4. 24개 매트릭스

아래 셀은 `cursor query p95 / build_page_tree p95`(ms)다.

| 포맷·입력 | 0ms | 80ms | 150ms | 250ms |
| --- | ---: | ---: | ---: | ---: |
| HWP 영문 | 44.5 / 44.5 | 45.1 / 45.0 | 52.6 / 52.5 | 65.2 / 65.0 |
| HWP 숫자 | 44.7 / 44.6 | 44.5 / 44.4 | 52.7 / 52.6 | 65.8 / 65.7 |
| HWP IME | 44.9 / 44.7 | 44.8 / 44.6 | 52.7 / 52.5 | 65.7 / 65.6 |
| HWPX 영문 | 44.5 / 44.4 | 44.4 / 44.3 | 52.9 / 52.8 | 65.6 / 65.6 |
| HWPX 숫자 | 44.5 / 44.4 | 44.5 / 44.3 | 52.6 / 52.4 | 65.1 / 65.0 |
| HWPX IME | 44.2 / 44.1 | 44.7 / 44.6 | 52.8 / 52.7 | 65.5 / 65.4 |

포맷·입력을 합친 cadence별 raw profile p95는 다음과 같다.

| cadence | profile 수 | Rust total p95 | build p95 | traversal p95 | char positions p95 | build 누적 비중 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 0ms | 200 | 44.6ms | 44.5ms | 0.0ms | 0.1ms | 99.87% |
| 80ms | 200 | 44.8ms | 44.7ms | 0.0ms | 0.1ms | 99.83% |
| 150ms | 200 | 52.8ms | 52.7ms | 0.0ms | 0.1ms | 99.88% |
| 250ms | 200 | 65.5ms | 65.5ms | 0.1ms | 0.1ms | 99.87% |

800개 전체에서 부가 구간 p95는 다음과 같다.

| 구간 | p95 |
| --- | ---: |
| `find_pages_for_paragraph` | 0.1ms |
| hint page 정렬 | 0.0ms |
| cache lookup | 0.0ms |
| page-tree clone | 0.1ms |
| cache store | 0.0ms |
| traversal inclusive | 0.1ms |
| traversal exclusive | 0.0ms |
| `compute_char_positions` | 0.1ms |
| WASM/bridge JSON 오버헤드 | 0.1ms |

각 query는 후보 115쪽을 얻지만 hint page 0을 먼저 시도해 41~42개 node, 20~21개 TextRun만
순회한 뒤 primary hit로 끝난다. 즉 후보 목록 길이나 선형 전체 페이지 탐색은 현재 타깃의 지배
비용이 아니다.

## 5. Stage 1 대조

Stage 1 normal API와 Stage 2 diagnostic API의 24개 cursor p95 차이는
`-2.1ms ~ +2.7ms`, 평균 `-0.1ms`, 평균 절댓값 `0.96ms`였다. 별도의 normal 모드 대조에서도
HWP 영문 cursor p95가 0ms `43.0ms`, 250ms `65.7ms`였고 diagnostic profile 수는 0이었다.
따라서 diagnostic wrapper가 관측한 cadence 관계는 기존 경로 범위 안에 있다.

## 6. 원인 판정

stable deferred 셀 편집은 `invalidate_page_tree_cache_from(0)`로 page 0 이후 cache를 비운다.
직후 caret 갱신의 exact path-near query는 정힌트 page 0을 바로 선택하지만 cache miss라
`build_page_tree(0)`을 동기로 수행한다. `build_page_tree`은 해당 페이지의 전체
`layout_engine.build_render_tree`와 부가 master-page/image 처리를 다시 실행한다.

이번 계측으로 확정된 범위:

- 편집 mutation 자체나 입력 종류는 지배 항이 아니다.
- 115쪽 후보 탐색과 tree traversal도 지배 항이 아니다.
- cache clone이나 `compute_char_positions`도 지배 항이 아니다.
- 편집마다 invalidated page의 전체 render tree를 cursor geometry 하나를 위해 재구축하는 것이
  #3137의 코드 수준 원인이다.
- 긴 cadence에서 build 자체가 더 느려지는 하드웨어 power/cache 상태 또는
  `build_render_tree` 하위 구간은 이번 Stage 2 범위에서 분리하지 않았다. 다만 전체 rebuild를
  제거하면 이 증가분도 cursor hot path에서 함께 제거되므로 구현 우선순위에는 영향을 주지 않는다.

## 7. 회귀 검증

| 검증 | 결과 |
| --- | --- |
| diagnostic native 좌표 동등성·cold miss/warm hit | 통과 |
| production WASM build | 통과 |
| Stage 2 전체 매트릭스 | 24/24, 800/800 profile |
| normal 모드 대조 | 2/2, diagnostic 0 |
| Rust `cargo test --lib` | 2,956 passed / 0 failed / 7 ignored |
| Studio `npm test` | 670 passed / 0 failed |
| Studio `npm run build` | 통과 |
| #2214 HWP focused/raw/delete/IME/save/print | 전 단계 통과 |

#2214 결과는 focused `flush=0`, boundary `steps=115`, stable operation p95 `46.7ms`,
save `flush → HWP export`, print `flush → 115 pages`를 유지했다.

## 8. 다음 단계

다음 구현은 page-tree cache를 무조건 재사용하는 방식이 아니라, stable edit의 focused
cell/line geometry를 edit revision에 묶어 재사용하는 Stage 3로 진행한다.

1. 로컬 reflow가 계산한 line/run 위치와 기존 cell/page 원점으로 caret rect를 만든다.
2. mutation 결과 또는 revision-scoped focused geometry cache로 caret 갱신에 전달한다.
3. 같은 revision·path·line의 stable 입력은 전체 `getCursorRectByPathNear`를 건너뛴다.
4. flow 경계, shadow pagination commit, 동기 flush, target/format/path 변경, revision 불일치는
   기존 exact query로 fallback한다.
5. 정상 좌표와 overflow clamp를 먼저 고정한 뒤 24개 매트릭스와 #2214 게이트를 다시 실행한다.

`with_page_tree_cached`로 clone만 없애는 변경은 p95 최대 0.1ms 항만 줄이므로 해결책이 아니다.
또한 exact query를 다음 frame으로 미루는 변경은 44~66ms rebuild long task의 위치만 옮긴다.
