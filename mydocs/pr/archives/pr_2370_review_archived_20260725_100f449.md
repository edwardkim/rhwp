---
kind: reference
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-25
---

# PR #2370 검토 기록 — undo P3 중복 emit 정리와 메인터너 보정

## 메타

| 항목 | 값 |
| --- | --- |
| 원 PR | [#2370](https://github.com/edwardkim/rhwp/pull/2370) |
| 작성자 | `lpaiu-cs` |
| 원 head | `ecdbc83af7cd2a795bd0fb84fd426dfce8a30849` (작성 시점 참고값) |
| base / 상태 | `devel` / `MERGEABLE`, `CLEAN`, 본문은 WIP 누적 트래커라고 명시 (모두 작성 시점 참고값) |
| 원 변경 | 1 파일, +3/-2 (`rhwp-studio/src/command/commands/insert.ts`) |
| 누적 검토 branch | `review/lpaiu-cs-20260725` (`upstream/devel` `efc4b6c4c58b696c2fd4d28bbb82cbfeb6e0499d`) |
| 적용한 contributor commit | `87f196b235c7e08f27b7a2c226e0a991f04841cc` → `98f7cd453` |
| maintainer 보정 | `fcfc966fa`, `40f2f4a3d` — contributor 원 변경과 분리 |

## 범위와 메인터너 지시 이행

원 변경은 회전·대칭 뒤 `recordObjectMutation()`이 snapshot의 `full` refresh → `afterEdit()` →
`document-changed`를 이미 발생시키는 상황에서 수동 `document-changed` emit 두 곳을 제거한다.
`executeOperation()`과 `afterEdit()` 호출 경로를 대조해 중복 render 신호 제거가 맞음을 확인했다.

[#2370의 메인터너 코멘트](https://github.com/edwardkim/rhwp/pull/2370#issuecomment-5011630445) 중
이번 누적 통합에 직접 필요한데 원 source에 없던 지시는 다음처럼 보정했다.

| 지시 | 반영 | 검증 |
| --- | --- | --- |
| #2375: Edit 필드 Escape는 취소여야 하며 blur가 뒤따라도 commit하지 않아야 함 | `showEditOverlay()` Escape 경로에서 `committed = true`를 overlay 제거보다 먼저 설정 | source guard와 실제 headless Chrome E2E: 값·undo stack 불변 |
| #2378: `removeCurrentField()`의 form-mode 분기는 현재 도달 경로 없는 방어 코드로 주석 정정 | `field:remove`의 `canExecute`와 경계 삭제의 form-mode bail을 주석·source guard에 명시 | `undo-field-ops.test.ts` |

다른 지시는 범위 밖으로 유지한다. `SnapshotCommand` 원자성(#2346)은 별도 이슈, 개체 조작 뒤 커서
착지(#2345)는 한컴 2022 실측 전 현행 유지, operation 인자 관례(#2368)는 신규 코드 관례만 적용한다.
`fieldMarker`/`exitKey` self-heal은 저비용 정리에 자연스럽게 포함될 때만 다룬다는 지시도 변경하지 않았다.

## 검증

통합 branch 전체 기준으로 다음을 통과했다. Cargo 계열은 검토 전용
`target/lpaiu-cs-20260725-review`, `CARGO_INCREMENTAL=0`에서 한 번에 하나씩 실행했다.

- `cargo build --release`
- `cargo test --release --lib` — 2,916 passed, 7 ignored
- `cargo test --profile release-test --tests` — 모든 실행 대상 통과
- `cargo test --profile release-test --features native-skia skia --lib` — 56 passed
- `cargo test --profile release-test --features native-skia --test issue_2225_missing_picture_placeholder` — 2 passed
- `cargo test --profile release-test --features native-skia --test render_p37_direct_pdf_export` — 4 passed
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --doc` — 4 passed, 2 ignored
- `(cd rhwp-studio && npx tsc --noEmit)`, `npm --prefix rhwp-studio test` — 637 passed
- `npm --prefix rhwp-studio run e2e:manifest-check` — tracked E2E 77개와 manifest 77행 정합
- `VITE_URL=http://127.0.0.1:7701 npm --prefix rhwp-studio run e2e:form-edit-escape` — 실제
  `form-01.hwp` Edit field에서 값 변경 → Escape → blur 뒤에도 값과 undo stack 모두 불변
- `wasm-pack build --target web --out-dir pkg`

## 판단과 후속 처리

**통합 PR로 수용 권고.** 원 #2370은 여전히 여러 P3 항목을 담은 살아있는 tracker이므로 이 한 commit을
수용했다고 원 PR을 자동 close하지 않는다. integration PR merge 전에는 그 PR의 최신 head CI와
작업지시자 승인을 다시 확인한다.

한컴 2022에서 Edit field Escape의 실제 동작이 취소와 다르면 #2375 보정만 되돌리고 결과를 별도
판정 코멘트로 남긴다. 현 단계에서는 GitHub comment·push·merge를 하지 않았다.

실행·rollback 순서는 [PR #2370 implementation 계획](pr_2370_review_impl.md)을 따른다.
