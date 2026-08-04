---
kind: reference
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-24
---

# PR #3223 검토 기록 — 문단 병합 undo 문단 메타 복원

## 메타

| 항목 | 값 |
| --- | --- |
| 원 PR | [#3223](https://github.com/edwardkim/rhwp/pull/3223) |
| 작성자 | `lpaiu-cs` |
| 원 head | `14161a8327386dd43f95cd18ac746573986bf56e` |
| base / 상태 | `devel` / `CONFLICTING`, `DIRTY` |
| 누적 검토 브랜치 | `review/lpaiu-cs-20260724` (`upstream/devel` `c8611dd84d002d2a776c040387bf21cf270f6448`) |
| 적용 순서 / 로컬 SHA | 7) `05b4dd807` → `8dbcb23ad`, maintainer 보정 `0cf1f98f6` |
| 충돌 | `src/wasm_api.rs` import 충돌만 해소; 양쪽 import 보존, 동작 충돌 없음 |

## 검토 결과

본문·머리말/꼬리말·각주/미주의 병합 역연산에서 `ParaMeta`를 보존하는 방향은 타당하다. 원 head의
P2는 `split_paragraph_native`가 square-OLE wrap chain에서 새 빈 문단을 만들 때만 `restore_meta`를
적용하지 않아, Backspace/Delete undo가 앵커 문단의 서식으로 되돌아가는 것이었다.

maintainer 보정 `0cf1f98f6`은 이 분기도 table-anchor·일반 `split_at` 경로와 같은 계약으로
`new_para.apply_meta(meta)`를 적용한다. 실제 `samples/한셀OLE.hwp`로 Enter → 빈 wrap 문단에 7개
ParaMeta 필드 설정 → Backspace merge → undo split을 수행하는 회귀를 추가해, `para_shape_id`,
`style_id`, break·numbering·raw header·tab metadata가 전부 복원되는지 확인한다.

## 독립 검증

| 검증 | 결과 |
| --- | --- |
| `cargo test --profile release-test --test issue_2069_ole_object_selection hwp_square_ole_merge_undo_restores_removed_paragraph_meta` | 1 passed |
| `cargo test --profile release-test --tests` | PASS (`ir_field_sweep_baseline` 포함) |
| `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --doc` | PASS (doc 4 passed, 2 ignored) |
| `wasm-pack build --target web --out-dir pkg` | PASS |
| `rhwp-studio: npx tsc --noEmit`, `npm test` | PASS (636 passed, 0 failed) |

모든 Cargo 검증은 `CARGO_INCREMENTAL=0`과 검토 전용
`CARGO_TARGET_DIR=target/lpaiu-cs-20260724-review`에서 직렬 실행했다.

## 최종 권고

**통합 PR 수용 후보.** 원 P2는 contract 누락을 보정하고 실제 OLE sample 회귀로 고정했다. 전체
Rust·Studio 검증과 통합 PR 최신 CI가 성공한 뒤 작업지시자 승인으로 반영한다.
