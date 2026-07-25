---
kind: reference
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-24
---

# PR #3213 검토 기록 — 머리말/꼬리말 필드 삽입 undo

## 메타

| 항목 | 값 |
| --- | --- |
| 원 PR | [#3213](https://github.com/edwardkim/rhwp/pull/3213) |
| 작성자 | `lpaiu-cs` |
| 원 head | `3ce1fb239efb093c4d392f1f4865a14e67bb76ce` |
| base / 상태 | `devel` / `BEHIND` |
| 누적 검토 브랜치 | `review/lpaiu-cs-20260724` (`upstream/devel` `c8611dd84d002d2a776c040387bf21cf270f6448`) |
| 적용 순서 / 로컬 SHA | 6) `8c34d63dd` → `8eec50e2b`, maintainer 보정 `0cab08c80` |
| 충돌 | 없음 |

## 검토 결과

PR은 머리말/꼬리말 안의 필드 삽입을 기록형 역연산 명령으로 남겨 undo 뒤에도 HF 편집 모드를 유지한다.
원 head의 P2는 파일명·두 자리 쪽번호처럼 field marker가 여러 표시 문자로 치환된 뒤 클릭하면, 렌더
offset이 모델 문단 끝을 넘어 `deleteTextInHeaderFooter` undo가 no-op이 되고 redo마다 marker가
누적되는 문제였다.

maintainer 보정 `0cab08c80`은 이를 mutation 계약 경계에서 해결했다. Studio는 삽입 전에
`getHeaderFooterParaInfo`의 **모델** `charCount`로 표시 offset을 정규화하고, Rust
`insert_field_in_hf_native`도 clamp 전 요청값이 아닌 실제 삽입 offset을 결과·이벤트에 반환한다.
따라서 history에는 존재하는 model offset과 양수 marker 길이만 기록된다. #3208 선행 변경은 이미
`upstream/devel`에 포함돼 있다.

새 Rust 회귀 `field_insert_reports_the_clamped_model_offset`은 파일명 marker 뒤에 표시 offset `7`을
전달해도 새 쪽번호 field의 반환 offset과 모델 charCount가 모두 `2`인지 고정한다.

## 독립 검증

| 검증 | 결과 |
| --- | --- |
| `cargo test --profile release-test --lib field_insert_reports_the_clamped_model_offset` | 1 passed |
| `cargo test --profile release-test --tests` | PASS (`ir_field_sweep_baseline` 포함) |
| `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --doc` | PASS (doc 4 passed, 2 ignored) |
| `wasm-pack build --target web --out-dir pkg` | PASS |
| `rhwp-studio: npx tsc --noEmit`, `npm test` | PASS (636 passed, 0 failed) |

모든 Cargo 검증은 `CARGO_INCREMENTAL=0`과 검토 전용
`CARGO_TARGET_DIR=target/lpaiu-cs-20260724-review`에서 직렬 실행했다.

## 최종 권고

**통합 PR 수용 후보.** 원 P2는 maintainer 보정과 회귀 테스트로 해소했다. 전체 Studio·Rust 게이트와
통합 PR 최신 CI가 성공한 뒤 작업지시자 승인으로 반영한다.
