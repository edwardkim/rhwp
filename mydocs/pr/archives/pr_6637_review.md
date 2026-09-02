---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-02
pr: 6637
issue: 6042
author: postmelee
---

# PR #6637 review - 다중 페이지 스크롤 렌더링 가상화

## 결론 - 승인, stack·review-only CI 대기

[PR #6637](https://github.com/edwardkim/rhwp/pull/6637)은 긴 다중 페이지 문서의 일반 scroll에서
visibility 판정과 여러 쪽 raster가 입력 callback을 연속 점유하던 경로를 row/X index, exact page
surface LRU, page-boundary scheduler로 분리한다. 사용자가 새 구간의 첫 내용을 더 빨리 볼 수 있게 하면서
동일 배율 warm 왕복은 완성 surface를 재사용한다. scroll 중에는 현재 surface를 유지하고 마지막 입력
150ms 뒤 visible 읽기 화질을 별도 safety gate 안에서 회복한다.

self-review 대상 code candidate는 `68beaa5dce0ac0fbc794761324abea959d0245ef`, 직접 base는 #6467
`23b5bcf73f6e8659a90b25ebfde1311e1965364f`다. 단계별 테스트·실문서 A/B에서 발견한 역방향 cache
thrash와 fractional DPR geometry 문제를 최종 후보에서 해소했고, 차단 finding은 남아 있지 않다.

이 PR은 Draft stack의 top이다. bottom #6458이 최신 `devel`과 충돌하므로 지금 Ready 또는 merge 후보로
올리지 않는다. 이 review-only 기록의 trailing head CI가 통과하고, bottom부터 stack을 restack한 뒤
descendant 전체를 재검증해야 한다.

## 검토 경로와 metadata

- 기본 경로: `collaborator_self_merge.md`
- 보조 경로: `local_validation.md`, `visual_fixture_evidence.md`, `review_only_fast_pass.md`,
  `rework_and_exceptions.md`
- self PR이므로 reviewer와 GitHub approval review를 지정하지 않는다.
- 작성 시점: OPEN / Draft, base `codex/issue-6041-budget-first-render-scale`,
  head `codex/issue-6042-page-virtualization`, `MERGEABLE/CLEAN`.
- code candidate 규모: 202 files, +489,954/-219, 21 commits. 이 중 제품·test·E2E는
  24 files, +4,063/-218이고, 162 files·약 16MB는 A/B raw evidence다.
- Rust source/test/fixture, Cargo, workflow 변경은 없다.

## 해결 범위와 사용자 변화

### 문제

#6467은 surface pixel 예산과 쪽별 DPR을 정하지만, 여러 쪽을 언제 판정·raster·보존할지는 기존 수명을
따른다. 페이지가 많은 문서를 저배율 다중 쪽 보기로 열고 먼 행으로 이동하면 전체 페이지 scan, visible
raster, 인접 prefetch가 한 입력 구간에 모일 수 있다. 같은 구간을 왕복할 때 완성 bitmap을 다시 그리는
경로도 있었다.

### 변경 후

- 먼 행 이동은 viewport 중심 visible부터 page 단위로 표시하고 각 쪽 사이에서 main thread에 양보한다.
- exact key가 같은 완성 main/overlay bundle은 physical-pixel 예산 안에서 LRU로 재부착한다.
- 새 scroll generation은 낡은 visible/prefetch를 기각하고, 선택 prefetch는 visible 완료 뒤 idle마다
  한 쪽만 처리한다.
- scroll 중에는 이미 표시된 bitmap의 DPR을 유지한다. 멈춘 뒤 충분히 노출된 visible만 center-first로
  화면 DPR을 회복한다.
- 초기 표시, zoom-settled, resize, edit/undo/redo, strict render와 focus/caret/ruler 의미는 기존처럼
  동기 처리한다.

## self-review 결과

### visibility와 좌표

- row index는 기존 layout이 만든 row top/bottom을 사용하고 후보 행에서 기존 strict AABB를 적용한다.
  mixed-size, facing 첫 blank slot, 마지막 미완성 행을 새 모델로 재해석하지 않는다.
- 모든 쪽이 한 row인 horizontal view는 X monotonic range로 별도 탐색한다. Y index만 적용해 전체 후보를
  누락하는 경로가 없다.
- snapshot key는 geometry revision과 scroll/viewport 네 값이다. geometry가 바뀐 같은 frame을 오래된
  결과로 처리하지 않는다.
- fractional DPR의 integer physical Canvas를 CSS 크기로 역산하지 않는다. main과 overlay 모두
  `PageInfo × zoom` logical box를 써 DPR 전환이 page/ruler/scroll geometry를 움직이지 않는다.

### surface 소유·예산

- page bundle exact key는 document/view scope, page, backend, layer 구성과 실제 render scale을 포함한다.
  zoom·revision·backend가 다른 bitmap을 재사용해 흐리게 표시하지 않는다.
- main뿐 아니라 background/behind/front overlay를 같이 detach/attach/dispose한다. 불완전 image/RawSvg
  후속 작업은 cacheable complete로 승격되지 않는다.
- active, LRU, anonymous pool, pending target reservation을 중복 없이 physical pixel로 센다. mandatory
  visible 자체가 예산을 넘으면 이를 숨기지 않고 optional cache/prefetch만 포기한다.
- target DPR 전환 전에 old actual 크기로 LRU를 trim하던 Stage 4 결함은 target-state reservation으로
  교정했다. 선택 prefetch는 admission과 dispatch 직전에 보존 이득을 다시 확인한다.

### scheduler와 취소

- 일반 scroll만 page scheduler에 연결한다. 1·2 visible initial과 strict/zoom/resize/mutation은 기존
  synchronous contract를 유지한다.
- 한 쪽 raster가 4ms를 넘더라도 중간 선점할 수 있다고 주장하지 않는다. page를 마친 뒤 다음 page 전에
  양보한다.
- visible queue가 끝나기 전에 prefetch하지 않고, idle deadline이 부족하면 다음 기회로 넘긴다.
- generation, desired exact key, document/view scope를 dispatch 직전에 다시 확인한다. 새 scroll, zoom,
  resize, mutation, reset, dispose와 renderer fallback이 낡은 callback을 취소한다.

### scroll-settled 화질

- active scroll planner에는 target이 아니라 DOM에 실제로 붙은 requested DPR을 lock한다. 예약만 된 DPR 2
  작업을 현재 surface로 오인하지 않는다.
- 마지막 입력 150ms 뒤 양 축 8 CSS px 이상 노출된 visible을 한 번 계산한다. raw 전체 비용이 64M
  이하면 화면 DPR, 넘으면 DPR 1.5를 시도하며 그조차 넘으면 #6467 결과를 유지한다.
- 64M은 retained cache 예산을 확대한 값이 아니다. settled-visible mandatory quality에만 쓰는 absolute
  guard다. print/high-quality profile은 기존 raw 경로다.
- viewport 쪽을 편집 focus로 바꾸지 않아 caret와 ruler 의미를 보존한다.

## 발견·교정한 문제

1. 최초 Stage 4는 active downscale이 끝나기 전에 old surface ledger로 exact LRU를 퇴거해
   `exam_kor` 역방향 raster를 3→5회로 늘렸다. Stage 5를 중단하고 target reservation과 optional
   prefetch gate를 추가한 뒤 3회로 복구했다.
2. probe가 admission에서 거절된 retained 후보까지 완료 대상으로 기다려 제품은 정착했지만 진단만
   timeout됐다. materialized retained set을 별도로 노출해 측정 의미를 교정했다.
3. scroll-settled DPR 1.5에서 integer physical size를 DPR로 역산하면 page box가 1px 미만 흔들렸다.
   logical CSS geometry를 직접 사용해 고정했다.
4. Puppeteer의 `Control+Home` 단일 key 문자열이 실제 DPR 1 E2E 시작 전에 실패했다. modifier down/press/up
   순서로 고치고 DPR 1 assertion을 추가했다. 제품 동작 변경은 아니다.

차단 finding은 모두 code candidate 전에 해결했고 해당 회귀를 결정론적/브라우저 테스트로 고정했다.

## 렌더 영향과 직접 시각 판정

이 PR은 문서 조판·paint 내용이나 한컴 출력 fidelity를 바꾸지 않고 화면 page Canvas의 수명·실행 순서를
바꾼다. 따라서 한컴 기준 PDF와의 layout visual sweep이 아니라, 같은 Studio fixture·viewport의 Stage 3/
code candidate A/B, DOM·surface snapshot, 실제 DPR을 직접 대조하는 경로를 선택했다.

review 작성 시 다음 6개 이미지를 직접 열어 확인했다.

| 역할 | 경로 | SHA-256 | 사람 판정 |
| --- | --- | --- | --- |
| Canvas2D 자동 34% before | `issue6042-stage5-expanded/canvas2d-exam-auto-34-stage3.jpg` | `209d5de3fbc11296b59a64246176d0f099ce07ce2193c8815a1a135230c48515` | 하단 3열 page grid·본문·ruler 확인 |
| Canvas2D 자동 34% after | `issue6042-stage5-expanded/canvas2d-exam-auto-34-corrected.jpg` | `cb9a5e20be43237113e23531f303f2e991903e3049edb016658702a473ab0121` | before와 큰 배치 이동·누락 없음 |
| 두 쪽 100% DPR 1 | `issue6042-stage5-scroll-quality/exam-double-100-before-dpr1.png` | `716b95b77dc117fbc2a900ccf1ede5b4f3f0c1dd7cbc8b31e48c4e119a005a28` | 같은 국어 영역·표·그림 crop 확인 |
| 두 쪽 100% DPR 2 | `issue6042-stage5-scroll-quality/exam-double-100-after-dpr2.png` | `4794d8b8f09e0c56a0a11aa5af07d18c2b63491eff693e60ab18c8cc2d321564` | 클릭 없이 같은 geometry에서 선명도 회복 |
| CanvasKit KTX before | `issue6042-stage5-expanded/canvaskit-ktx-100-stage3.jpg` | `3f37a11a5965e0bdb7069d0a8b985567f507ee34aacd7d2b8da7d7935f4472bb` | probe가 상단을 가리고 나머지가 흰 면이라 fidelity 근거로 부적합 |
| CanvasKit KTX after | `issue6042-stage5-expanded/canvaskit-ktx-100-corrected.jpg` | `fee20c6205a88753c6bd68224309b481ebeaaefe6faa377f621b42775ff556c6` | before와 동일 한계, 기계 surface/image state만 근거로 사용 |

경로의 공통 prefix는 `mydocs/working/assets/`다. 자동 34% JPEG의 상단 절반도 probe JSON이 가리므로
glyph fidelity 수치로 쓰지 않았다. 실제 내용이 보이는 하단 page grid에서 배치·공백·누락만 확인했고,
기계 비교는 0.0592% 차이와 동일 DOM/surface snapshot을 보고했다. 두 쪽 crop은 같은 내용·box에서 실제
DPR 1→2 승격을 보여 주지만 선명도 개선률을 pixel diff로 주장하지 않는다. KTX screenshot은 위 한계 때문에
시각 수용 근거에서 제외하고 flow image ready, integer layer surface, pending/error 0만 사용했다.

이것은 외부 contributor PR이 아니므로 merge 후 contributor comment 계획은 해당하지 않는다. PR 본문에는
현재 branch의 안정 경로로 Canvas2D·두 쪽 대표 이미지를 직접 표시했다.

## 성능 판정

Chromium 151, 1280×720 CSS px, 실제 DPR 2, Canvas2D의 교대 A/B다. 경보선은 결과 확인 전에 p50
`max(10ms, 5%)`, p95 `max(25ms, 10%)`로 고정했다.

- 178쪽 4열·34% cold 20쌍: 첫 visible 271.2/278.9ms → 56.5/59.0ms,
  `visibility.update` 243.8/247.7ms → 19.1/19.6ms.
- 같은 cold 표본: main raster 12회 동일, long task 40건·6,887ms → 0건.
- 178쪽 warm: retained 16.9/17.7ms → 16.6/17.7ms, main raster 양쪽 0회.
- `exam_kor` 역방향 보정: raster 3/3, cache take 4/4. retained +4.2/+4.6ms로 경보선 안.
- Canvas2D·CanvasKit 확장 성능 표본 revision당 260개: complete, error 0, 사전 경보선 안.
- 정착 화질: scroll callback 0/0.1ms 동등. DPR 1→2 추가 raster로 final known work는
  +33.3/+54.1ms, active surface는 +14,702,520px. 화질 복원을 위한 의도한 후처리 비용이다.

첫 visible 수치를 모든 장치의 frame 개선으로 일반화하지 않는다. long task는 PerformanceObserver 값이지
compositor dropped-frame 수가 아니다. LRU는 메모리를 더 보존해 CPU 재작업을 줄이는 교환이며 전체
메모리 감소도 주장하지 않는다.

## 시각·수명 검증

- Canvas2D, CanvasKit, auto fallback
- 자동 1↔2↔3열, 한 쪽·두 쪽·4열, facing, horizontal, 마지막 미완성 행
- 34/50/100/200% zoom과 반복 좌표, 빠른 방향 반전
- `exam_kor`, `hwpspec`, `kps-ai`, 실제 4쪽, 21쪽 다층, KTX 4-layer 문서 교체
- edit→undo→redo의 네 invalidation scope
- 실제 DPR 1 Chrome의 28개 viewport resize snapshot
- embedded image decode 3회 실패 뒤 fallback 3회, 완료 서명·queue·error 잔류 0

사용자는 Stage 3/현재 로컬 서버를 직접 비교하고 scroll-settled 화질 보정 뒤 앞서 제보한 버벅임이
사라진 것 같다고 확인했다. 주관 관찰은 자동화 근거의 보조로만 사용했다.

## 로컬 검증

- Studio: 1,422 total / 1,421 pass / 1 policy skip / 0 fail
- TypeScript `--noEmit`, Vite production build: passed
- E2E manifest: 126/126
- scheduler/LRU/CanvasView/budget/visibility/zoom focused suite: 71/71
- headless Chrome DPR 1 ruler/resize: 28/28
- headless Chrome image decode failure/fallback: passed
- Stage 집계기 4종, raw JSON parse, markdown link, secret pattern, ancestry, `git diff --check`: passed
- Rust source/test/fixture/Cargo/CI 변경 없음: Rust gate N/A

## 잔여 위험과 범위 밖

- 한 쪽 raster 내부는 선점할 수 없어 복잡한 단일 쪽은 여전히 한 task를 길게 점유할 수 있다.
- settled-visible 64M은 단순 RGBA 약 256MB에 해당하며 GPU 복사·임시 Canvas는 포함하지 않는다. 무조건
  allocation하는 값이 아니라 raw/1.5/fallback 판정의 상한이지만, 장치별 적응은 후속 범위다.
- 짧은 문서 전체 surface 상주, 장치 성능 자동 확장, worker/OffscreenCanvas, 저배율 DPR 정책은 포함하지
  않았다.
- raw evidence 162개가 GitHub diff 노이즈를 만든다. reviewer는 최종 report와 summary, 24개
  source/test/E2E부터 확인할 수 있고 raw는 재집계 근거로 보존한다.
- bottom #6458 conflict를 해소한 뒤 #6467과 이 PR을 restack하면 최신 head에서 전체 자격을 다시 확인한다.

## 현재 판정

- 판정: **승인**
- 코드·로컬 검증: 통과
- 차단 finding: 없음
- 원격 상태: Draft 유지
- 남은 조건: review-only trailing head push와 CI, bottom-first restack, descendant 재검증, 별도 Ready 승인
- 원격 조치: self PR이므로 GitHub approval review는 만들지 않는다. issue close·Ready·merge도 수행하지
  않는다.
