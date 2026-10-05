---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-05
---

# PR #7578 리뷰 — fix: respect explicit PasswordChar when rendering Edit forms

## 최종 판정

**머지 보류 — 누적 후보 검증 진행 중.** 원 head의 CI와 이번 누적 head의 실행 결과를 구분한다. 필수 코드·회귀·시각 검증 결과를 확인한 뒤 판정을 갱신한다.

## 접수 정보

- 원 PR: [#7578](https://github.com/edwardkim/rhwp/pull/7578), semanticist21, devel 대상, non-draft.
- 원 head: `e3268b9bc5aba217e51b3104a87b7a5f19f6491d`; 접수 시점 `MERGEABLE` / `CLEAN`. 원 head의 상태이며 누적 후보 판정이 아니다.
- 누적 branch: `review/semanticist21-20261005`; 고정 base `cdba77b609c399fdef26a6c9e637716aa32c2177`; 누적 code candidate `1d809afe7b965c9ea6137012d59d139d63b038d0`.
- Reviewer: jangster77 지정. 기본 maintainer_general; intake_and_review, local_validation, multi_pr_update_branch, 렌더 영향 시 visual_fixture_evidence를 적용.
- 관련 이슈: PR 본문과 실제 변경 범위에서 추가 확인 필요.
- 사용자 지시: non-draft 19건을 번호 순으로 누적 체리픽; 충돌은 메인터너 보정; 원 PR별 리뷰 기록을 개별 작성.

## 적용 이력

| 원 commit SHA | 상태 | 로컬 적용 SHA | 메인터너 보정 |
| --- | --- | --- | --- |
| `c3bb8a72cdd1dc933b35c141b9033692a48dc8a2` | applied | `86ed33a8f50c6b36693cc091bb872ea62829e201` | — |
| `6bf112ac0d53ce9d958ea571cf3822e571102553` | applied | `58582b7f94cef0d432ae77bd4de059f06ab84fa3` | — |
| `350852e1139eaa8a1f9135b2c05521edc44ea6db` | applied | `0084c8a7330760959f247c0e6c3f1cb9c391da41` | — |
| `30817e498364ec61ee3c43c9c2a88a1ed4c471be` | applied | `eb49764d6292967a659ecb474b000368b160933c` | — |
| `e3268b9bc5aba217e51b3104a87b7a5f19f6491d` | applied | `1d809afe7b965c9ea6137012d59d139d63b038d0` | — |

원 저자와 `cherry-pick -x` 출처를 보존했다. 이미 patch-id가 같은 원 commit은 중복 적용하지 않았다. 원 contributor branch는 수정하지 않았다.

## 변경·소비 경로 검토

- `src/paint/builder.rs`
- `src/paint/json.rs`
- `src/render_backend/scenes.rs`
- `src/renderer/canvas.rs`
- `src/renderer/canvaskit_policy.rs`
- `src/renderer/layout/paragraph_layout.rs`
- `src/renderer/layout/shape_layout.rs`
- `src/renderer/render_tree.rs`
- `src/renderer/skia/renderer.rs`
- `src/renderer/svg.rs`
- `src/renderer/web_canvas.rs`

구현 주장과 실제 호출 경로·반례 대조: 진행 중. 합성 입력의 계약 결과를 한컴 출력과의 일치 증거로 바꾸지 않는다.

## 검증 입력·결과

- `tests/cases/form_password_rendering.rs` (4개 테스트): 실행 대기

- 로컬 Cargo는 공유 `target/pr-review`에서 순차 실행한다. 같은 원 head의 CI를 19건 누적 후보의 전체 검증으로 재사용하지 않는다.
- 필수 fmt·Clippy 3종·workspace build·manifest/base 정책, focused·전체 Rust, 해당 Native/fresh WASM 검증: 진행 중.
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
