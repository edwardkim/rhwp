---
kind: pr-review-implementation
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-02
pr: 6637
issue: 6042
---

# PR #6637 구현 검토 - 다중 페이지 scroll 작업·surface 수명 분리

## 제출 계보

1. #6467 head `23b5bcf73`을 직접 base로 계획과 Stage 1 관찰 계약을 고정했다.
2. Stage 2에서 기존 geometry를 보존한 row/X index와 immutable visibility snapshot을 연결했다.
3. Stage 3에서 main·overlay 완성 bundle의 exact LRU와 active/LRU/pool physical-pixel ledger를 연결했다.
4. Stage 4에서 일반 scroll visible을 page-boundary rAF slice로 나누고 optional prefetch를 bounded idle로
   이동했다.
5. Stage 5 A/B에서 역방향 cache thrash를 차단 finding으로 잡아 확장 matrix를 중단했다.
6. target-state reservation과 prefetch admission/dispatch benefit gate로 thrash를 해소하고, probe의
   materialized-retained 완료 의미를 교정했다.
7. 사용자 피드백에 따라 scroll 중 실제 surface DPR lock과 150ms settled-visible quality promotion을
   추가하고 fractional DPR CSS geometry를 고정했다.
8. 실제 DPR 1, 28 viewport resize, image decode failure/fallback, Canvas2D·CanvasKit·auto, 실제 문서
   확장 matrix와 사용자 직접 조작을 통과했다.
9. code candidate `68beaa5dc`를 push하고 #6467 branch를 base로 Draft PR #6637을 생성했다.

## 보호 불변식

- page layout과 zoom anchor 계산은 #6040 결과를 유지한다.
- DPR 후보·32M visible/40M retained budget은 #6467이 계속 소유한다.
- visibility index는 기존 geometry와 AABB 의미를 재해석하지 않는다.
- exact key가 다르거나 image/RawSvg 후속 작업이 미완료된 surface는 재사용하지 않는다.
- mandatory visible은 budget 때문에 건너뛰지 않고 optional prefetch/cache만 제한한다.
- scroll callback에서는 현재 surface를 유지하며 settled quality raster는 별도 rAF page work다.
- focus, caret, selection, hit-test, ruler update와 initial/zoom/resize/mutation strict 경로는 기존 의미를
  유지한다.
- document/view generation이 바뀐 callback은 새 화면에 결과를 게시하지 않는다.

## 검토 초점

- `virtual-scroll.ts`: row/X 탐색 경계, mixed/facing/last-row, snapshot invalidation
- `page-surface-lru.ts`·`canvas-pool.ts`: bundle ownership, exact key, trim/dispose, ledger 중복
- `page-render-scheduler.ts`: page priority, soft budget, generation, rAF/idle/timer 취소
- `canvas-view.ts`: sync/async 진입 분리, exact attach/detach, target reservation, settled quality
- `render-surface-budget.ts`: active DPR lock과 64M raw/1.5/planner fallback
- `page-renderer.ts`: image completion과 actual bundle key
- browser E2E: DPR 1 resize/ruler, image failure fallback과 queue 정착

## 성능·화질 판단

긴 문서 cold 이동의 핵심 이득은 raster 횟수 감소가 아니라 **같은 작업의 입력 callback 밖 분할과 첫
visible 우선순위**다. warm 왕복은 exact cache hit로 raster 0을 유지했다. 정착 visible의 DPR 2 회복은
성능 이득이 아니라 사용자가 읽을 때 화질을 되돌리는 후처리이며, +33.3/+54.1ms와 +14.7M active pixel을
명시적으로 보고했다.

Stage 5에서 나온 불리한 `exam_kor` 역방향 결과를 숨기지 않고 구현을 보정한 뒤 같은 조건으로 재수용했다.
기준선보다 retained p50/p95가 +4.2/+4.6ms 늦지만 사전 경보선 안이며 raster/cache hit가 기준선과 같다.

## 검증 자산

- 최종 report: `mydocs/report/task_m100_6042_report.md`
- Stage 5 failure: `mydocs/working/task_m100_6042_stage5.md`
- correction: `mydocs/working/task_m100_6042_stage4_correction.md`
- expanded matrix: `mydocs/working/task_m100_6042_stage5_expanded.md`
- settled quality: `mydocs/working/task_m100_6042_stage5_scroll_quality_correction.md`
- final Stage 5/6: `mydocs/working/task_m100_6042_stage5_complete.md`,
  `mydocs/working/task_m100_6042_stage6.md`
- raw/summary/screenshots: `mydocs/working/assets/issue6042*`

## 다음 조건

1. 이 review-only 기록과 당일 orders를 trailing commit으로 PR #6637에 push한다. **완료**
2. 최신 trailing head의 base/head, Draft, 게시 본문을 재확인한다. **완료**
3. #6458·#6467·#6637을 trunk `devel`의 native stack #6640으로 연결하고 1/3·2/3·3/3 등록을 API로
   확인한다. **완료**
4. native stack 규칙상 protection과 Actions가 trunk `devel` 기준으로 평가됨을 기록하고, top의 후속
   push가 실제 CI를 시작하는 것을 확인한다. **완료**
5. bottom #6458 conflict를 cascading rebase로 해소해 세 layer의 선형성을 회복한다. **완료**
6. 갱신된 각 exact head의 Studio unit과 top TypeScript·build·E2E manifest를 재확인한다. **완료**
7. 갱신된 각 exact head의 Full CI와 #6040/#6041/#6042 시각·성능 게이트를 재확인한다.
8. 세 PR을 일괄 Ready로 바꾸는 것은 작업지시자의 별도 승인 뒤 수행한다.
