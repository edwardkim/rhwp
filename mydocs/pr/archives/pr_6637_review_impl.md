---
kind: pr-review-implementation
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-04
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

## 2026-09-04 인라인 리뷰 보정 계획

- 대상: review `5109298479`, 원 head `777fba96ef437c3f865653e6f96d13a3d0312317`.
- 승인: 사용자의 "보정하고 보정 코멘트까지 게시해줘"에 따라 두 finding의 수정·검증·PR branch push와
  인라인 답글 게시를 수행한다. merge와 추가 정책 변경은 포함하지 않는다.
- 기존 Stage 6 이후의 제한된 보정 루프다. 이미 완료한 이슈 단계를 새로 채번하지 않는다.

1. RED: 실제 CanvasView planner/descriptor와 scheduler를 연결해 정착 큐 생성 뒤 focus 변경으로
   미생성 visible 쪽이 유실되는 조건을 고정한다. visible frame/fast path/prefetch의 run 및 validity
   예외가 남은 작업 예약을 잃는 조건도 회귀 테스트로 남긴다.
2. 구현: focus 변경으로 plan을 교체하면 남은 desired work와 선택 prefetch 예약을 최신 plan으로
   다시 계산한다. 같은 viewport generation의 frame·scroll-settle 예약은 보존하고 추가 동기 visible
   raster를 강제하지 않는다. scheduler는 예외를 숨기지 않으면서 finally에서 후속 dispatch를 보장하고,
   동기 fast path 전에 scroll-settle을 예약한다.
3. GREEN: focused tests, TypeScript, Studio 전체 테스트, production build, E2E manifest 및 실제
   브라우저의 focus/scroll/zoom·오류 복구를 검증한다. Rust/WASM source와 32M/40M/64M 예산,
   DPR 후보, 줌 앵커, 캐시 키 정책은 변경하지 않는다.
4. 결과와 한계를 review 기록에 추가하고 현재 PR head에 fast-forward push한다. 각 인라인 리뷰에
   수정 commit·재현/검증 결과를 답글로 게시하고 API로 원문을 재확인한다. 새 head의 원격 CI는
   로컬 검증과 구분해 확인하며, 사용자 승인 전 merge하지 않는다.

### 보정 실행 결과

- 1~2 완료: RED 12 fail을 재현한 뒤 queue 재구성·예외 후속 예약·dispatch 전 settle 예약을 구현했다.
- 3 완료: focused 34/34, 전체 Studio 1,449 total / 1,448 pass / 1 skip, TypeScript·build·manifest
  127/127 통과. 실제 `exam_kor.hwp`의 34/50/100% scroll·click smoke도 누락/queue/error 0이었다.
  계획의 오류 복구는 결정론적 fault injection으로 확인했으며 브라우저 invariant throw 재현은 하지
  않았다. 전체 성능 A/B의 재측정도 이번 제한된 보정에는 포함하지 않았다.
- 4의 push·답글과 새 head CI는 원격 실행 결과로 확인한다. 세부 재현·수용 근거 및 한계는
  [review 기록](pr_6637_review.md)의 2026-09-04 보정 절을 따른다.

### 2026-09-04 기준선 정리·로컬 사용자 검증

- 사용자 추가 수정 head `1c9b5245e217f6b4a6da4b8ceba7eb2c402423b8`을 보존한다. DPR plan 재래스터
  실패 쪽을 `discardActivePageSurface`로 회수하는 #6467 인접 결함 수정이며, 별도 정책 변경은 아니다.
- 사용자의 후속 진행·push·서버 실행 승인에 따라 최신 `devel@a1be9d49313002a42dbca3ec5c03529c00dd6a4b`을
  merge commit으로 통합한다. 원 PR 커밋을 재작성하지 않으며 자동 병합 외 source 보정은 하지 않는다.
- 할일 문서의 다른 작업 기록과 archive 위치를 보존하고, 이 PR의 이동 후 상대 링크·현재 상태만 정리한다.
- 통합 source의 WASM·Studio 로컬 검증 뒤 push하고 새 head를 실행하는 loopback 서버를 제공한다.
  사용자가 CI 완료 대기를 생략하도록 지시했으므로 원격 CI 통과나 merge 완료로 보고하지 않는다.

실행 결과: `9a52b09d229b474b5ac268a1a41fcca9673fa496`으로 충돌 없이 통합했다. Studio 전체
1,437 pass / 1 policy skip, TypeScript·production build·E2E manifest 127/127 및 문서 검사를
통과했다. 새 WASM을 사용하는 `4198` 서버에서 `exam_kor.hwp` 20쪽의 100% 문서 열기 smoke와
브라우저 오류 0건을 확인했다. native `--no-opt` 빌드이므로 정량 성능 비교용은 아니며, 사용자
스크롤·줌 확인과 새 head의 원격 CI 및 merge 승인은 별도 조건으로 남긴다.

## 2026-09-04 검증 자료 최소화 계획

- 사용자 승인 범위: 개발 패널 사용 안내와 PR 핵심 증거만 남기는 로컬 정리. **실제 push는 별도 승인 후**
  수행하며 PR 본문 게시·merge·ready 상태 변경은 하지 않는다.
- 기준은 원격 rebase 후 `9b679f07a8b714d680ed822406e41cc62a6174ea`다. 로컬
  `codex/pr-6637-evidence-trim`에서 작업하고 이전 `codex/pr-6637-inline-review-fixes`는 보존한다.
- 제품·패널·테스트 코드는 그대로 둔다. 매뉴얼은 기존 DEV opt-in UI, 실제 조작, JSON 저장과 지표의
  한계를 설명한다. 패널 일반화나 별도 PR 분리는 포함하지 않는다.
- 162개 자산 중 34개를 원문 그대로 남긴다. cold 우선 표시, warm/reverse 재사용, 두 쪽에서 지연이
  증가한 사례, 정착 화질/추가 비용에 필요한 모든 A/B 반복과 대표 이미지를 선택한다. 선택한 시나리오
  안에서 유리한 반복만 추리지 않는다. 중복·폐기 표본·초기 smoke·불필요한 집계기는 제거한다.
- 역사적 summary와 실패 원인 보고는 유지하되 재계산 가능한 범위를 색인과 검사기로 명시한다. 과거
  측정 결과를 위 최신 head의 신규 측정으로 표기하지 않는다. 별도 영구 원시 archive는 만들지 않는다.
- 검증: 보존 파일 SHA-256/JSON parse, 주요 수치 재집계, 남은 집계기 입력, 삭제 경로 참조, Markdown
  링크/메타데이터의 신규 오류 0건, 제품 diff 없음, `git diff --check`. 정리 전 메타데이터 오류는 이
  작업과 무관한 기존 4문서·16건이다. 새 성능 측정이나 제품 CI를 대신하는 검증이 아니다.

### 증적 정리 실행 결과

- 계획대로 중간 자산 128개(이미지 10개 포함)를 제거했다. 기존 34개를 byte-identical로 보존하고
  색인·manifest·read-only 검사기 3개를 추가해 자산은 162→37개, 17.35→약 5.18MB로 줄었다.
- 새 [개발 패널 안내](../../manual/studio_scroll_probe_guide.md)는 DEV opt-in, 조작, JSON 수동 저장,
  계측 on/off와 제품 A/B의 차이, cold/warm 조건과 비교 한계를 설명한다. manual 지도에서 연결했다.
- 보존 파일 34개의 SHA-256/bytes/JSON, 핵심 104개 p50/p95 계열, cold long-task 합계와 quality
  delta를 검산했다. correction 원 집계기의 전체 summary/ledger/verdict도 byte-identical 재생성됐다.
  무결성 위반·요약값 변조·반복 누락은 메모리 내 fault injection으로 기대한 guard에서 거부됨을 확인했다.
- quality의 `settledKnownWorkMs`가 trace `retainedComplete`를 가리킨다는 것을 명확히 했다. 추가 안정
  프레임을 기다린 runner의 `knownWorkNextFrameMs`와는 다른 값이며 기존 보고 수치를 바꾸지 않았다.
- 삭제 자료 참조 49개는 정리 전 commit에 실제 target이 존재하는지 확인했다. 변경 문서의 내부 상대
  링크 오류는 0건, 장기 문서 메타데이터 신규 오류는 0건(기존 16건 유지)이다. 제품 코드 diff는 없다.
- 기존 PR 본문 초안에 최소 증거/사용 안내와 측정 revision·범위 설명을 반영했다. 원격 PR 본문은
  게시하지 않았으며 **push 전 사용자 승인 대기**다. 제거 파일은 정리 전 commit에서 복구할 수 있다.
