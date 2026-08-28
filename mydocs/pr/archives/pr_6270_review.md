# PR #6270 self-review 기록

## 대상

| 항목 | 값 |
| --- | --- |
| PR | [#6270](https://github.com/edwardkim/rhwp/pull/6270) |
| 작성자 | `edwardkim` |
| base / source | `devel` / `task_m100_4969` |
| 최초 제출 head | `c0ad765a2e804a41c9f98065be597575b6ebf217` |
| registry 보정 code candidate | `a271733ba50ca9c1c81481326208eb0dc3e3fc75` |
| registry 보정 작성자 | `jangster77` |
| 기준 `upstream/devel` | `5645e1f5bbd8cd28380aa153a29f4ddcf58c5405` |
| 규모 | 문서 작성 시점 81 files, +12,356 / -99, 13 commits |
| 상태 | code candidate CI 완료 시점 `MERGEABLE` / `CLEAN`; trailing head와 merge 직전 재확인 필요 |

관련 이슈는 [#4969](https://github.com/edwardkim/rhwp/issues/4969)이며 상위 추적은
[#4960](https://github.com/edwardkim/rhwp/issues/4960)이다. 이 PR은 Q2-D4까지의 bounded horizontal common
shaping을 제출하며 D5와 vertical/RTL/variation 확대는 포함하지 않는다. 따라서 PR 본문은 `Refs #4969`를
사용하고 이슈를 자동 close하지 않는다.

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
  `review_only_fast_pass.md`, `rework_and_exceptions.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서
- current head: `a271733ba50ca9c1c81481326208eb0dc3e3fc75`

## 검토 범위

- exact font source identity와 bounded GSUB/GPOS request, terminal disposition ledger가 font bytes·raw text·host
  path를 trace에 남기지 않는지 확인했다.
- cluster-aware horizontal measurement가 paragraph transaction, composition `Arc`, emitted run mapping,
  page-local `NodeId` sidecar와 common GlyphRun lowerer까지 한 owner 계보로 전달되는지 대사했다.
- one-line/one-run/direct old-Hangul/embedded exact source/left-aligned compressed ratio인 최초 lane만 원자적으로
  활성화하고, preflight·source identity·generation·affine·backend 증명 중 하나라도 실패하면 TextRun을
  보존하는지 확인했다.
- CanvasKit/CanvasKitBrowser는 strict common GlyphRun을 선택하고 Canvas2D·legacy SVG·native Skia는 증명되지
  않은 경로에서 TextRun fallback을 유지하는지 확인했다.
- Q0~D4 계획·단계 보고서·기계 판독 JSON, 공개 Source Han/Happiness Sans fixture와 D4-C 성능 보고서를
  구현·테스트와 교차 확인했다.
- 최신 `devel` 병합으로 유입된 #6172/#6184/#6185/#6186 renderer 변경과 #4969 공통 파일의 hunk가 겹치지
  않고 양쪽 focused 회귀가 함께 통과하는지 확인했다.

## collaborator registry 보정

PR 생성 뒤 `jangster77`가 commit `a271733ba`에서 module harness와 호환되지 않는 shaping test source 7개를
`RHWP GENERATED TEST TARGETS` marker block에 singleton target으로 등록했다. 이 변경은 일반 generated suite나
manifest를 제출한 것이 아니라, maintainer registry 예외 target을 동기화한 것이다. 실제로 보정 전 review
worktree의 `--prepare`는 `Cargo.toml generated test target block drift`로 중단됐고, 보정 뒤에는 sync 없이
prepare/check가 통과했다. 기여와 authorship을 그대로 보존한다.

## 발견 사항

### 차단 결함

현재까지 코드 차단 결함은 발견하지 못했다. strict activation subset 밖의 입력은 기존 layout·TextRun으로
닫히고, common GlyphRun 게시 실패가 부분 geometry나 font resource를 남기지 않는다.

### PR 본문 정정 필요

최초 PR 본문의 체크리스트에는 Cargo generated target을 포함하지 않았다고 적혀 있다. `a271733ba` 이후에는
singleton exception target 7개의 marker block을 포함하므로 이 문장은 사실과 다르다. merge 전 본문을
“generated suite·manifest는 미포함, maintainer registry marker block만 동기화”로 정정해야 한다.

### 범위 밖·잔여 위험

- multi-line/run, mixed target, center/right/justify/distribute, identity·expansion ratio, RTL/vertical,
  variation replay, nonzero GPOS y positioning은 D4 subset이 아니며 fail-closed 또는 TextRun fallback이다.
- native Skia는 blob typeface construction 증명이 없으므로 common GlyphRun을 선택하지 않는다.
- exact portable font 456,688B를 새 layer resource에 등록·JSON 직렬화하는 비용이 크다. page/document 수준
  resource·digest·JSON 재사용을 입증하기 전에는 activation lane을 확대하지 않는다.
- 이 PR은 1,000줄을 넘는 대형 PR이다. 단계별 승인 commit과 기계 판독 결과가 있더라도 최신 head CI,
  self-review trailing head와 메인테이너 merge 승인을 별도 cycle로 확인하며 즉시 admin merge하지 않는다.

## 로컬 검증

최신 `devel@5645e1f5bb`를 포함한 최초 제출 후보 `c0ad765a2`에서 다음을 완료했다.

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --all` / `cargo fmt --all -- --check` / `git diff --check` | PASS |
| Rust unit-tier | 4,221 tests / 299 modules / violation 0 |
| review worktree manifest | 995 sources / 4,468 attrs / 32 suites + 16 exceptions / 48 targets, PASS |
| #4969 focused integration | 46 PASS |
| #5821 및 최신 devel renderer 교차 회귀 | 6 PASS |
| Rust main library | 3,889 PASS / 13 ignored |
| auxiliary Rust libraries | 182 PASS |
| native Skia proof/fallback | 15 PASS |
| native·WASM lib clippy | PASS |
| Node CI 정책·trusted reuse 계약 | 77 PASS |
| Python workflow 계약 | 166 PASS |
| Docker WASM release build | PASS, 5분 44초 |
| Studio unit | 1,221 PASS / 1 skip |
| Studio production build | PASS |
| CanvasKit #4969 actual draw·font coverage·renderer contract | PASS |

registry 보정 후보 `a271733ba`에서는 review worktree `--prepare`와 `--check`가 sync 없이 통과했고,
새 singleton target 7개의 44개 테스트가 모두 통과했다. 보정은 Cargo target registry만 바꾸므로 위 code candidate의
전체 제품 회귀와 CanvasKit pixel 결과를 다시 귀속하지 않고, 최신 GitHub Full CI로 최종 후보를 검증한다.

## CI

code candidate `a271733ba50ca9c1c81481326208eb0dc3e3fc75`에 귀속된 다음 GitHub run은 모두
`completed / success`다.

| workflow | run | 결과 |
| --- | --- | --- |
| CI | [33143739457](https://github.com/edwardkim/rhwp/actions/runs/33143739457) | SUCCESS |
| CodeQL | [33143739477](https://github.com/edwardkim/rhwp/actions/runs/33143739477) | SUCCESS |
| Render Diff | [33143739316](https://github.com/edwardkim/rhwp/actions/runs/33143739316) | SUCCESS |
| Adapter inter-diff | [33143739461](https://github.com/edwardkim/rhwp/actions/runs/33143739461) | SUCCESS |
| Proptest roundtrip | [33143739488](https://github.com/edwardkim/rhwp/actions/runs/33143739488) | SUCCESS |

이 review와 오늘할일만 single-parent trailing commit으로 추가한다. push 뒤 최신 head의 review-only fast-pass
preflight와 required aggregate 성공을 다시 확인한다.

## 시각·오라클 증적

이 PR은 특정 HWP/HWPX 문서 페이지나 기준 PDF의 개선을 주장하지 않는다. 공개 Source Han subset의 고정 glyph
ID·cluster·position·advance를 실제 CanvasKit software surface에서 draw해 pixel bbox를 직접 읽는 E2E가 해당
activation lane의 시각 오라클이다.

- 현재 `sqrt(ratio)` 계약: 58×34px ink bbox
- #5821 이전 비교 계약: 58×38px ink bbox
- page advance: 61.824px, local→page affine mismatch 0px
- strict false 또는 malformed affine: TextRun fallback

이 결과는 메인테이너가 D4-C 단계에서 확인·승인했다. 문서 전체 visual fidelity로 과장하지 않으며, 실제 문서
오라클이 필요한 multi-line/vertical/variation 확대는 후속 승인 범위로 남긴다.

## 성능 판정

동일 공개 fixture·release probe에서 D4-A dormant 대비 D4-B active 결과는 다음과 같다.

| 항목 | dormant | active | 차이 |
| --- | ---: | ---: | ---: |
| warm layer median | 0.843µs | 703.600µs | +702.757µs |
| cold layer median | 1.608ms | 2.314ms | +0.706ms / +43.91% |
| layer JSON | 6,849B | 619,562B | +612,713B |
| portable font payload | 0B | 456,688B | +456,688B |

최초 strict lane의 정확성과 fail-closed 조건은 충족하지만 비용은 미미하지 않다. 따라서 현재 subset은 유지하고
resource 재사용 증명 전에는 lane을 확대하지 않는 `qualified-bounded` 판정을 유지한다.

## 결론

현재 self-review 결론은 **정상 merge 후보**다. 코드와 로컬 검증에서 차단 결함은 발견되지 않았고
`a271733ba` registry 보정도 필요한 예외 target 동기화로 확인됐으며 exact code candidate의 GitHub Full CI가
모두 성공했다. PR 본문을 현행화하고 review-only trailing head의 required checks와 최신 mergeability를
확인한 뒤 정상 merge를 권고한다. merge와 #4969 후속 상태 변경은 각각 메인테이너 승인 후 수행한다.
