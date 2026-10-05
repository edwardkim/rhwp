---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-05
---

# PR #7504 리뷰 — 수정: #7503 커닝한 글자를 커닝 전 폭으로 그리고, 셀 캐럿과 입력 비용을 바로잡는다

## 최종 판정

**머지 보류 — 누적 후보 검증 진행 중.** 원 head의 CI와 이번 누적 head의 실행 결과를 구분한다. 필수 코드·회귀·시각 검증 결과를 확인한 뒤 판정을 갱신한다.

## 접수 정보

- 원 PR: [#7504](https://github.com/edwardkim/rhwp/pull/7504), semanticist21, devel 대상, non-draft.
- 원 head: `7dc340284bd84e2ee475da3b577146005b989500`; 접수 시점 `MERGEABLE` / `CLEAN`. 원 head의 상태이며 누적 후보 판정이 아니다.
- 누적 branch: `review/semanticist21-20261005`; 고정 base `cdba77b609c399fdef26a6c9e637716aa32c2177`; 누적 code candidate `1d809afe7b965c9ea6137012d59d139d63b038d0`.
- Reviewer: jangster77 지정. 기본 maintainer_general; intake_and_review, local_validation, multi_pr_update_branch, 렌더 영향 시 visual_fixture_evidence를 적용.
- 관련 이슈: #7503.
- 사용자 지시: non-draft 19건을 번호 순으로 누적 체리픽; 충돌은 메인터너 보정; 원 PR별 리뷰 기록을 개별 작성.

## 적용 이력

| 원 commit SHA | 상태 | 로컬 적용 SHA | 메인터너 보정 |
| --- | --- | --- | --- |
| `38ce705edd33c656faf7f5831a19ed8da3864370` | applied | `993f4bc14af375c8abd59d902ace954c775d39d8` | — |
| `8345ad79ab4334198ae195b417ef53c09bd09e54` | applied | `00c42120cc779e9f2855346ec5c3593cdccb763f` | — |
| `71fd3348ffb2125176cbf9ac8b7e38243f6cc285` | applied | `bcc5efc1142f65d9d712cf31654d803561b2e4d2` | — |
| `e9f38a16b24c59151c2f721a02172f9329b86675` | applied | `279607f753e44d9c0cc7fae12540d1a279f02558` | — |
| `7dc340284bd84e2ee475da3b577146005b989500` | applied | `d66a26cdcb09a58b9b341cd8f0b5b8166519749b` | — |

원 저자와 `cherry-pick -x` 출처를 보존했다. 이미 patch-id가 같은 원 commit은 중복 적용하지 않았다. 원 contributor branch는 수정하지 않았다.

## 변경·소비 경로 검토

- `src/document_core/commands/text_editing.rs`
- `src/renderer/kerning.rs`
- `src/renderer/mod.rs`
- `src/renderer/svg.rs`
- `src/renderer/web_canvas.rs`

구현 주장과 실제 호출 경로·반례 대조: 진행 중. 합성 입력의 계약 결과를 한컴 출력과의 일치 증거로 바꾸지 않는다.

## 검증 입력·결과

- `tests/cases/issue_7503_kerning_glyph_caret_cost.rs` (5개 테스트): 실행 대기

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
