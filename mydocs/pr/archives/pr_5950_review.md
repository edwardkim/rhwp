---
kind: pr-review
status: self-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-23
---

# PR #5950 self-review — W7 canonical font registry와 backend projection

## 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
  `review_only_fast_pass.md`, `rework_and_exceptions.md`의 대형 PR 경로
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서
- 작성자 본인 self-review이므로 reviewer를 지정하지 않는다.
- code candidate: `bbfd3ad6de7e80797dddc8bf690616573beb6e7d`

## metadata

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#5950](https://github.com/edwardkim/rhwp/pull/5950) |
| 작성자 | `edwardkim` |
| 관련 이슈 | [#4966](https://github.com/edwardkim/rhwp/issues/4966), parent [#4960](https://github.com/edwardkim/rhwp/issues/4960) |
| base / head | `devel` / `task_m100_4966` |
| 규모 | 54 files, +103,047 / -931, 14 commits |
| 상태 | Draft, `MERGEABLE`, `mergeStateStatus=CLEAN` |

1,000줄을 크게 넘지만 canonical registry·migration baseline JSON 약 76,000행과 Rust·TypeScript 정적
projection 약 19,000행이 대부분이다. handwritten runtime은 Rust style·metric lookup과 Studio loader·
substitution·trace adapter로 한정된다. 대형 PR 규칙에 따라 생성물은 눈으로 전수 비교하지 않고 frozen
baseline, generator의 byte 대사와 runtime 전수 계약으로 검토했다.

## 목적과 변경 범위

#4966의 목적은 Rust layout, Canvas2D paint, webfont supply와 CanvasKit SFNT에 흩어진 유한 폰트 규칙을
830개 canonical registry에서 결정면별 정적 projection으로 생성하되 기존 선택 결과와 렌더 출력을 바꾸지
않는 것이다.

- 한 rule은 relation·decision plane·projection 하나만 소유한다.
- Rust는 legacy-Latin → HFT → TTF 이름 치환 우선순위와 metric alias 뒤의
  exact → bold-only → name-first 사다리를 유지한다.
- Studio는 기존 substitution chain, 정부상징 successor, webfont와 CanvasKit 공급 capability를 보존하고
  실제로 관여한 generated `ruleId`를 Decision Trace에 연결한다.
- schema 1.0의 830개 population은 read-only이며 mapping 변경·retirement는 후속 schema 승인 대상이다.
- font binary, metric 값 교정, fallback 정책 확대와 private corpus 자료는 범위 밖이다.

## self-review findings

### [P1][해결] canonical 이관 뒤 source-side test oracle이 이중 authority로 남았다

최초 PR head `4a7c0f431`은 `font_metrics_data.rs`와 `style_resolver.rs`에 신규 test support 6개를 추가했고,
PR-base unit-tier가 이를 정확히 거부했다. `745660467`은 helper 위치를 기존 test module 안으로 옮겼지만
제품 `src/** #[cfg(test)]`에 신규 수기 mapping oracle을 남겨 현행 `CONTRIBUTING.md`의 의미 계약을 여전히
충족하지 못했다.

`fc2194b2c`에서 해당 helper와 source unit 3건을 제거하고
`tests/cases/issue_4966_font_rule_projection.rs` 공개 API integration 2건으로 옮겼다. layout-name 171개 중
137개 runtime 도달 rule과 우선순위상 shadowed인 HFT 34개를 구분하고, metric alias 67개 × bold/italic
4조합과 미등록 sentinel을 검사한다. 최종 PR-base unit-tier는 4,221 tests / 299 modules, 신규 support
증가 없이 통과했다.

### [P2][해결] prepared worktree의 fmt 결과가 source commit에 전파되지 않았다

W7-R4 정리 중 prepared review worktree의 `cargo fmt --all`이 신규 integration 원본을 포맷했지만 최초 로컬
코드 commit에는 그 결과가 반영되지 않은 것을 발견했다. 삭제 전 worktree diff를 회수해 코드 commit에
fixup했고, 검증 worktree의 source tree와 `fc2194b2c`가 일치함을 확인했다. 최종 GitHub Lint의
`Format check`도 같은 head에서 성공했다.

### 추가 blocker 없음

- `font_rule_registry.mjs check`, `font_rule_projection_gen.mjs check`,
  `font_rule_projection_baseline.mjs check`가 canonical bytes와 현재 5개 projection을 다시 대사해 통과했다.
- generated source의 fixed allowlist·whole-file ownership, atomic paired output과 stale/manual edit 거부
  계약을 확인했다. handwritten runtime이나 임의 경로를 generator가 덮어쓰는 경로는 없다.
- Rust 미등록 face는 원명 또는 `None`을 유지하고, generated rule ID가 provenance와 다르면 trace가
  fail-closed한다. Studio의 supply capability는 Canvas2D 등록과 CanvasKit SFNT 계획을 혼동하지 않는다.
- registry·projection·runtime snapshot의 수량·hash 및 공개 integration 결과가 최종 보고서와 일치한다.

## 렌더·시각 증적 판정

renderer·WASM 경계가 바뀌므로 `visual_fixture_evidence.md`를 적용했다. 이 PR은 사용자-visible 개선이 아니라
출력 불변을 주장한다. 동일 source에서 만든 native와 Docker WASM으로 공개 HWP 7문서 167쪽 및 대표
HWP/HWPX 6문서 page 0을 비교해 총 173쪽 SVG byte mismatch 0을 확인했다. 별도 fixture·golden·기준 PDF
변경이 없고 사람의 시각 판정을 merge 근거로 사용하지 않았으므로 대표 review PNG는 추가하지 않는다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| W1·W2·W3·W6·W7 Node contract | 87/87 통과 |
| registry·projection·pre-migration baseline check | 통과 |
| source Rust focused | 35/35 통과 |
| 공개 API integration | 2/2 통과 |
| PR-base unit-tier | 4,221 tests / 299 modules, drift 없음 |
| release library | 4,071 pass / 13 ignore |
| release-test nextest | 8,200/8,200 pass / 41 skip |
| Native Skia 공식 3종 | library 4,128 pass / 13 ignore, 2/2, 4/4 |
| Clippy·rustdoc·fmt·diff·Markdown link | 모두 통과 |
| Studio | TypeScript, 1,070 pass / 1 skip, production build 통과 |
| Docker WASM·fresh Decision Trace | optimized build 통과, 3/3 |
| native/WASM 공개 parity | 13문서 173쪽, mismatch 0 |

분리 worktree의 최초 Decision Trace 실행은 `rhwp-studio/node_modules`가 없어 `@noble/hashes`를 해석하지
못했다. 제품 실패로 분류하지 않고 기존 dependency 설치를 연결한 뒤 같은 fresh Docker WASM에서 3/3을
통과했다. 검증에는 private corpus를 사용하지 않았다.

## GitHub Actions

code candidate `bbfd3ad6d`의 [CI run 32644553674](https://github.com/edwardkim/rhwp/actions/runs/32644553674)는
Lint, Native Skia, Frontend package, test archive builder·worker와 Build & Test가 모두 성공했다.
[CodeQL 32644553651](https://github.com/edwardkim/rhwp/actions/runs/32644553651),
[Render Diff 32644553606](https://github.com/edwardkim/rhwp/actions/runs/32644553606),
[Proptest 32644553629](https://github.com/edwardkim/rhwp/actions/runs/32644553629),
[Adapter inter-diff 32644553616](https://github.com/edwardkim/rhwp/actions/runs/32644553616)도 같은 SHA에서
성공했다. 정책상 WASM Build와 Frontend unit gates의 skip 외에 실패한 필수 check는 없다.

현재 self-review·오늘할일은 이 녹색 code candidate 뒤의 `mydocs/` 한정 single-parent trailing commit이다.
push 뒤 review-only fast-pass의 candidate 재사용과 최신 aggregate 성공을 확인해야 한다.

## 최종 권고

최초 unit-tier 실패의 원인을 baseline 완화나 runtime 수기 표 복원으로 우회하지 않고 공개 integration
경계로 정정했다. 생성 authority, runtime 우선순위, fallback·metric·renderer 출력 불변과 최신 Full CI가
일치하며 추가 blocker는 발견하지 않았다. self-review는 **완료 / 조건부 merge 권고**다. trailing
review-only head의 fast-pass, Draft 해제 승인, 최신 `MERGEABLE/CLEAN`과 메인테이너의 정상 merge commit
방식 병합 승인을 각각 확인하기 전에는 merge하지 않는다.
