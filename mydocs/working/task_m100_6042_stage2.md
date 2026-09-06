# Task M100 #6042 Stage 2 — 행/X 인덱스와 visibility snapshot

- Issue: [#6042](https://github.com/edwardkim/rhwp/issues/6042)
- 완료: 2026-09-02 13:06 KST
- 상태: **Stage 2 구현·검증 완료, Stage 3 미착수**
- 직접 base: #6467 `23b5bcf73f6e8659a90b25ebfde1311e1965364f`
- Stage 2 시작점: `1466d79eb34426caedd80a1f7b7eb937ddc0c54d`
- 계획: [수행](../plans/task_m100_6042.md), [구현](../plans/task_m100_6042_impl.md)

## 1. 구현 범위

`VirtualScroll.setPageDimensions`의 쪽 배치 산식은 바꾸지 않았다. 산출된 `rowPages`, page offset·크기·left를
읽어 row top/bottom과 가로 left/right 탐색 인덱스를 만들었다. 세로는 첫 `rowBottom > viewportTop`과
첫 `rowTop >= viewportBottom`을 이진 탐색하고, 가로는 strict X 교차 후보를 이진 탐색한다. 후보에만
기존 strict AABB predicate를 적용한다.

`geometryRevision + scrollX + scrollY + viewportWidth + viewportHeight`를 key로 최근 불변 snapshot 하나를
보존한다. 결과에는 visible pages/rows, 첫·끝 행, 인접 prefetch, 조사 rows/pages가 들어간다. 기존
`getVisiblePages`·`getPrefetchPages`는 snapshot 배열을 복사하는 호환 wrapper로 남겼다. `CanvasView`는
visible과 prefetch를 같은 snapshot 한 번으로 확정한다.

`getPageAtY`·`getRowFirstPageAtY`·`getPageAtPoint`도 같은 행/X 인덱스를 쓰되, 첫 행 위 좌표, `NaN`·무한 X,
page gap의 동률에서 왼쪽 쪽을 고르는 기존 선형 fallback을 보존했다. 문서 `reset()`은 geometry와 cache를
먼저 비워 늦은 viewport-scroll이 이전 문서 page를 다시 요청하지 못하게 한다.

이번 단계에서 LRU, render scheduler, DPR·surface 예산, zoom anchor, ruler/caret/selection 시점, Rust/WASM은
변경하지 않았다. DEV 관찰기는 새 snapshot 경계와 실제 후보 page 수를 세도록 이름만 맞췄다.

## 2. 정확성 검증

- single/double/facing/multiple/auto/horizontal, mixed width/height, 음수 scroll, 0-width sentinel,
  viewport 경계에서 새 snapshot을 기존 전 페이지 선형 AABB·prefetch 오라클과 비교했다.
- hit test는 배치별 `x/y = -∞, -1, 0, 8, 250, 650, 1200, 3000, +∞, NaN` 조합을 이전 선형 참조와 비교했다.
- 같은 key는 같은 frozen object를 반환하고 legacy wrapper 반환 배열의 외부 변경은 cache를 오염시키지 않는다.
  같은 좌표라도 geometry 재계산·reset 뒤에는 revision이 증가한 새 snapshot을 반환한다.
- 500쪽 합성 geometry에서 세로 4열은 최대 16쪽/4행, 가로는 최대 6쪽/1행만 조사했다. 이는 알고리즘
  상한 확인용이며 실문서 성능 개선 수치로 쓰지 않는다.
- 실제 `CanvasView.reset()` 실행 테스트에서 기존 page geometry가 0쪽으로 바뀌고 후속 viewport query가
  empty visible/prefetch를 반환했다.

## 3. 현재 restack before/after 브라우저 대조

[요약 JSON](assets/issue6042-stage2/browser-ab.json)은 Canvas2D, 1280×720 CSS px, DPR 2,
`samples/hwpspec.hwp` 178쪽, 자동 4열·34%에서 같은 `다음 행` 세 번을 각각 실행한 자료다.

| 항목 | 수정 전 `1466d79eb` | Stage 2 | 변화 |
| --- | ---: | ---: | ---: |
| interaction당 visibility 계산 | linear 2회 + prefetch wrapper 1회 | snapshot 1회 | visible/prefetch 단일 결과 공유 |
| strict predicate 대상 | 356 page-unit (`178×2`) | 후보 12쪽 | **344 감소, 96.63%** |
| 세 interaction update time | 20.5 / 32.1 / 13.4ms | 20.3 / 27.8 / 12.9ms | n=3 탐색값, 개선률 주장 안 함 |
| visible | 8–19쪽 | 8–19쪽 | 동일 |
| retained | 4–23쪽 | 4–23쪽 | 동일 |
| probe/console 오류 | 0 / 0 | 0 / 0 | 동일 |

조회 자체는 두 쪽 모두 대부분 0.0–0.1ms 분해능에 머물렀고 전체 update는 raster가 지배한다. 따라서
96.63%는 **페이지 predicate 조사량 감소**이지 스크롤 체감 시간 개선률이 아니다. end-to-end 성능은 후속
LRU/scheduler 단계와 Stage 5의 최소 20회 교대 표본 없이 주장하지 않는다.

## 4. 실문서 수명·줌·시각 확인

- 178쪽에서 4쪽 실문서로 교체한 뒤 상태표시는 `1 / 4 쪽`, main Canvas 4개가 모두 visible이었다.
  Stage 1에서 원본에도 재현한 `[CanvasView] 페이지 N 정보가 없습니다`와 다른 warning/error는 0건이었다.
- 자동 배치의 표시 묶음 중심과 편집 영역 중심의 차이는 50% 3열 **0.0703px**, 100% 1열 **0.0078px**,
  34% 4열 **0.0664px**였다. 줌 왕복 뒤 열 수와 중앙 정렬을 유지했다.
- [178쪽 자동 34% JPEG](assets/issue6042-stage2/hwpspec-auto-34.jpg)는 실제 compositor 배치 확인용이다.
  JPEG이므로 화질 동등성·픽셀 차분 근거로 사용하지 않는다. 이 단계는 render scale을 바꾸지 않았다.

## 5. 검증 게이트

- 최신 restack 구현 전 집중 회귀: **74/74**
- Stage 2 집중 회귀: **79/79**, 추가 오라클군 포함 재실행 **36/36**
- 전체 Studio: **1,390 tests / 1,389 pass / 1 skip / 0 fail**
- `npx tsc --noEmit`: 통과
- `npm run build`: 통과, 247 modules. 기존 CanvasKit `fs/path` externalization과 large chunk 경고 기록
- `npm run e2e:manifest-check`: tracked 125 / manifest 125, 통과
- `git diff --check`: 통과
- 로컬 generated WASM SHA-256:
  `9a18f5638bf3550a8ea148cd6d296d0d2fb3a6378e9982b680366f985e7f9a09`; 추적 변경 아님
- Rust source/test/fixture 변경 없음. Rust lint bundle 대상이 아니다.

## 6. 인계와 다음 승인

Stage 2의 정확성·중복 query 제거·문서 전환 경계는 수용 기준을 충족했다. 인덱스만으로 체감 향상을
약속하지 않으며, 이 결과 때문에 화질·surface 예산을 바꾸지 않았다.

다음 승인 대상은 **Stage 3: 완료된 page surface bundle의 정확한 key·pixel ledger·LRU·무효화**다.
승인 전 LRU/scheduler를 구현하지 않고, #6042 branch push·Draft PR 생성, 하단 PR Ready/merge도 하지 않는다.
