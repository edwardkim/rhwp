---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-05
---

# PR #7577 리뷰 — 수정: 본문 그림·도형 삭제 후 이웃 누름틀과 활성 입력을 보존한다

## 최종 판정

**머지 보류 — 누적 후보 검증 진행 중.** 원 head의 CI와 이번 누적 head의 실행 결과를 구분한다. 필수 코드·회귀·시각 검증 결과를 확인한 뒤 판정을 갱신한다.

## 접수 정보

- 원 PR: [#7577](https://github.com/edwardkim/rhwp/pull/7577), semanticist21, devel 대상, non-draft.
- 원 head: `2e3eeb52892541ee0089818972a7b48f7e93fcd4`; 접수 시점 `MERGEABLE` / `CLEAN`. 원 head의 상태이며 누적 후보 판정이 아니다.
- 누적 branch: `review/semanticist21-20261005`; 고정 base `cdba77b609c399fdef26a6c9e637716aa32c2177`; 누적 code candidate `1d809afe7b965c9ea6137012d59d139d63b038d0`.
- Reviewer: jangster77 지정. 기본 maintainer_general; intake_and_review, local_validation, multi_pr_update_branch, 렌더 영향 시 visual_fixture_evidence를 적용.
- 관련 이슈: PR 본문과 실제 변경 범위에서 추가 확인 필요.
- 사용자 지시: non-draft 19건을 번호 순으로 누적 체리픽; 충돌은 메인터너 보정; 원 PR별 리뷰 기록을 개별 작성.

## 적용 이력

| 원 commit SHA | 상태 | 로컬 적용 SHA | 메인터너 보정 |
| --- | --- | --- | --- |
| `643467dd8633c2d3131e85d25e6a33f2dba0e0ce` | applied | `657b9f50039fd33e680925f93a861cb6aaa39286` | — |
| `6065fd827d811c8a8e837c694db0617a1d05beaf` | applied | `c1dae69e64f3db470d39046685b53a1d3bf0e993` | — |
| `767b5ad185da3d05c2b6f84eb8adced747dd8015` | applied | `6288a4e7fa8afd662898a24d5a07f3dcf2961880` | — |
| `3439c1ccf925ad7ddcfeb59547a6e3ec386496fb` | applied | `09f327d01a6bf515859b7306c451274f2a06100b` | — |
| `2e3eeb52892541ee0089818972a7b48f7e93fcd4` | applied | `4f6c4921e954eed83b0b90afe130b7e317414843` | — |

원 저자와 `cherry-pick -x` 출처를 보존했다. 이미 patch-id가 같은 원 commit은 중복 적용하지 않았다. 원 contributor branch는 수정하지 않았다.

## 변경·소비 경로 검토

- `src/document_core/commands/object_ops/picture.rs`
- `src/document_core/commands/object_ops/shape.rs`

picture/shape controls.remove 뒤 FieldRange.control_idx와 같은 본문 active field control_idx를 옮긴다. cell_path 있는 active field는 본문 필드 보정 대상으로 취급하지 않는다.

public API 생성 후 HWP 저장/재열기로 유효 필드가 있음을 먼저 확인한다. 개체 앞뒤·빈/채운 필드·snapshot 및 활성 필드 끝 입력을 검사한다. 셀 path 자체의 outer control index 재기준화는 이 수정 범위 밖이다. 합성 입력의 계약 결과를 한컴 출력과의 일치 증거로 바꾸지 않는다.

## 검증 입력·결과

- `tests/cases/delete_object_field_ranges.rs` (8개 테스트): 누적 head 실행 8 PASS / 0 FAIL

- 로컬 Cargo는 공유 `target/pr-review`에서 순차 실행한다. 같은 원 head의 CI를 19건 누적 후보의 전체 검증으로 재사용하지 않는다.
- 필수 fmt·Native/WASM/workspace-all-targets Clippy·workspace build·manifest/base 정책·source unit tier 검사: PASS. 전체 Rust·Native Skia·fresh WASM 및 직접 시각 검증은 진행 중.
- focused: `focused-command.json`의 20 case 필터, 전체 74건 중 73 PASS / 1 FAIL. PR별 결과는 위 case 항목에서 구분한다. 실행 증거: `output/pr-review/semanticist21-20261005/logs/focused.log`.
- 실제 HWP/HWPX/PDF 입력과 commit의 해시, 독립 기준, source/build provenance: 입력 사용 시 기록한다.
- source 교정 또는 검사 실패가 생기면 이 PR의 보정과 재실행을 별도로 기록한다. Golden/baseline/래칫을 완화하지 않는다.

## 원 head CI 참고값

- Lint (fmt, clippy, WASM check): SUCCESS
- Build & Test: SUCCESS
- CI Impact Policy: SUCCESS

## 조판·시각 판정

적용 여부와 필요한 직접 증거를 확인 중이다. 원 PR 제공 before/after·수치를 누적 head의 Visual Sweep 통과로 간주하지 않는다. 자료 부족과 실제 회귀를 구분하여 미검증/미충족으로 판정한다.

## 남은 범위·후속 처리

원 PR 전체 해결 여부와 이슈 종료 표현은 직접 검증한 범위로 제한한다. 이 기록은 로컬 누적 검토이며 원격 approve/comment/merge를 의미하지 않는다. 통합 결과는 같은 누적 branch에 두고 원 PR별 판정이 확정된 뒤 게시 범위를 결정한다.
