---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-27
---

# PR #7446 리뷰 — #7443 후속 표 식별·캐시 보정

## 최종 판정

**승인 — 로컬 검증 및 최신 head의 GitHub Full CI가 통과했다.**
원 PR #7443은 이미 승인·병합됐다. 이 판정은 후속 PR #7446에만 적용한다.
검증한 PR head는 `189ff38304263694cdd7fe8b1b399dd53b99b872`다.
사용자는 2026-09-27 문서 push, 새 head CI 확인 후 병합·후속 처리·작업 산출물 정리를 승인했다.
문서-only commit 뒤 새 head의 preflight·필수 aggregate와 mergeability를 확인한 다음 일반 경로로 병합한다.
`--admin`은 사용하지 않는다.

## 접수 정보

- [후속 PR #7446](https://github.com/edwardkim/rhwp/pull/7446), 작성자 @postmelee, base `devel`.
- base: `013bc846fca3434ccb1e4167744bed9440c6da6a`; code candidate: `9e1cffccfc9401262fff806f16b388180d5a663c`.
- code commit은 이전 로컬 보정 `7af493cb68a6dd94f6dc087e723d44b0738b7939`의 cherry-pick이다.
  전체 tree `98aee4a4a45a1b4d9e11582fb644845f9f9f6d77`가 동일하다. 이후 commit은 리뷰·증적 기록이다.
- base route: `collaborator_self_merge`; modifiers: `intake_and_review`, `local_validation`.
  원 PR 병합 후 처리는 `post_merge`를 적용했다. 자신에게 GitHub APPROVE를 제출하거나 reviewer를 지정하지 않는다.

## 변경과 검증

다른 구역의 동일 번호 표를 선택 대상으로 받지 않도록 section을 비교하고, 같은 중첩 표 안의 leaf 셀·문단
이동에는 bbox cache/failure memo를 재사용한다. 모든 조상 경로와 마지막 control은 구분한다.
세 회귀 테스트는 보정 전 FAIL/후 PASS였다. 기존 13개와 관련 cache 검사까지 21개도 앞선 검토에서 통과했다.
Rust grid 검사는 문단 불일치를 skip하지 않고 실제 bbox 페이지·구역·안쪽 표 경로를 확인한다.
모든 좌표가 정확한 leaf 셀과 일치해야 한다는 가정은 기존 텍스트 우선 계약과 다르므로 넣지 않았다.

최종 code candidate에서 다음을 재실행하고 통과했다.

- `cargo fmt --all -- --check`, native·WASM32 lib·workspace all-target Clippy(`-D warnings`), workspace build.
- `node scripts/rust-test-suite-manifest.mjs --check --base-ref 013bc846fca3434ccb1e4167744bed9440c6da6a`.
  파생 suite는 PR에 포함하지 않았다. source-side unit test 변경은 없어 unit-tier 검사는 비해당이다.
- `node scripts/run-rust-test.mjs issue_7442_nested_cell_hit_test -- --cargo-profile release-test --target-dir target/pr-review`: 7/7.
- `npm --prefix rhwp-studio test`: 1,803/1,803. TypeScript 포함 `npm --prefix rhwp-studio run build`와 E2E manifest 검사도 통과했다.
- 실제 Chrome E2E: F5 선택·드래그·부모 모델 보존·Undo·빈 영역 hit·일반 드래그·블록/개체 선택 PASS.

[명령별 결과·해시](../assets/pr7443_review/followup-validation.json)와
[원 PR 리뷰의 검증 결과](pr_7443_review.md#검증-결과)를 근거로 삼는다.
이전 동일 tree의 전체 Rust 결과 및 로컬 파일명 문제에 대한 재검사 내역은 원 PR 리뷰에 그대로 보존했다.
이번 diff는 Rust 제품 소스·snapshot·baseline을 바꾸지 않는 test helper와 Studio 변경이므로
전체 Rust 회귀를 중복 실행하지 않고 해당 lint·focused 및 Studio 검증을 수행했다.

## 입력·조판 원칙·미검증

[원본 HWP](../../../samples/basic/issue1994_behindtext_table_20200830.hwp)는 검토 commit에 포함되어 있고
실행 바이트의 SHA-256 `8e7a95cf591944bff56050879fa90251921ec57e28eac66d40c6fb8ad103016f`가 일치한다.
Rust/Cargo 입력이 기존 fresh WASM 빌드와 동일하며 `pkg`/Studio public WASM SHA-256도 일치했다.
WASM `91d8dfb9e44560338b1915fe1e6554b15278fe819b11ebe58feb8fdcc052909a`를 재사용했다.

조판 원칙·Native Skia·Visual Sweep: **비해당**. 이미 생성된 hit의 식별과 bbox cache key를 바꾸며
레이아웃·측정·배치·paint·분할을 생성하는 코드를 변경하지 않는다.
[비교 영상](../assets/pr7443_review/f5-resize-before-after.mp4)은 원 PR 이전과 보정 포함 버전의 UI 증거이며,
후속 보정만의 시각 변화 또는 한컴 출력 일치 증거는 아니다. 두 번째 7×4 중첩 표 문서는 미특정·미검증이다.

## 원 PR 후속 처리와 유지 범위

[원 PR 최종 코멘트](https://github.com/edwardkim/rhwp/pull/7443#issuecomment-5855597661)와
[이슈 종료 확인 코멘트](https://github.com/edwardkim/rhwp/issues/7442#issuecomment-5855597818)를 게시하고 본문을 API로 확인했다.
원 기여자의 commit·fork branch는 보존한다. 로컬 devel은 merge SHA로 동기화했다.

검토에는 기본 작업공간, `codex/pr-7443-followup`, 기존 보정 참조 branch,
`output/pr-review/7443/before` 비교 worktree, 원 head 비교 서버와 검증 로그를 사용했다.
사용자 승인에 따라 병합·필수 후속 처리 뒤 작업 전용 서버·branch·worktree·로그·임시 산출물을 정리한다.
리뷰 문서·영상·검증 요약은 이 PR에 보존하며 공유 `target/pr-review`와 다른 작업의 worktree는 보존한다.

## GitHub CI 완료 후 collaborator 병합 조건 확인

- 검증 head `189ff38304263694cdd7fe8b1b399dd53b99b872`와
  [Full CI run](https://github.com/edwardkim/rhwp/actions/runs/36317358388)의 `headSha`가 일치하며 결과는 `success`다.
- Rust A/B/C/D, lint, Frontend package, CodeQL 언어별 분석, Render Diff, Adapter, Proptest와 CI Impact Policy가 통과했다.
  GHAS CodeQL 집계의 neutral 및 영향 범위상 skipped job을 실패로 해석하지 않는다.
- `gh pr checks --required`가 반환한 필수 `Build & Test`는 PASS다. 실패·진행 중인 검사는 없다.
- PR은 OPEN, non-draft, MERGEABLE/CLEAN이며 source는 `edwardkim/rhwp`다. code candidate 이후 diff는 mydocs 기록뿐이다.
- @postmelee 권한은 push=true/admin=false다. review 요청·reviewDecision 차단이 없고 GitHub 병합 상태는 CLEAN이다.
  상세 branch-protection API는 권한 제약으로 404였으며, 조회 가능한 branch ruleset은 빈 목록이다.
  이 사실을 모든 보호 규칙 부재로 해석하지 않는다. 현재 PR의 필수 체크 조회와 병합 판정을 근거로 삼는다.
- 본인 PR에 GitHub APPROVE를 제출하지 않는다. collaborator self-review 문서가 PR diff에 포함돼 있다.
- 두 번째 문서 미검증은 원 PR 승인 때 명시한 기존 범위 한계이며 이번 보정의 새 blocker는 아니다.

## 병합 후 기록 계획

후속 PR에 새 head CI 판정, 실제 merge SHA, 로컬 검증·이슈 종료 유지 상태를 기록한다.
원 PR #7443과 이슈 #7442에는 후속 보정까지 병합됐다는 링크를 덧붙인다.
상세 리뷰·영상은 이번 PR head에 모두 포함돼 있으므로 종료 기록만을 위한 별도 PR은 만들지 않는다.
최종 devel 동기화와 해당 작업 산출물 정리를 확인한 뒤 완료 보고한다.
